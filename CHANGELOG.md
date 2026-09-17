# Changelog

## 0.3.0 (unreleased)

Replaces the hand-maintained pool list with on-chain discovery, and fixes a set
of correctness and availability bugs found by repeated adversarial review. Read
the **Deploy checklist** at the bottom before shipping.

### Headline

The registry goes from **61 hand-maintained pools to 271** (33 hand-maintained
plus 238 auto-discovered). It is now generated from three SPL-family stake-pool
programs and committed for review, instead of pubkeys being found by hand.

Every one of the 61 existing pool names still resolves and fetches, verified
against the live production API. See **Compatibility**.

### Added

- **Pool discovery generator** (`cargo run --features discover --example
  discover_pools -- --min-sol 1`). Enumerates the SPL, Sanctum SPL and Sanctum
  SPL Multi stake-pool programs, derives each pool's withdraw authority, names
  it from its LST mint, and rewrites `src/pools.rs` for human review. Behind the
  optional `discover` feature, so it reaches no consumer of the library.
- **Strict fetch variants**: `fetch_pools_strict`, `fetch_all_pools_strict`,
  `fetch_all_pools_with_stats_strict`. All-or-nothing. The existing lenient
  methods return `Ok` when at least one pool succeeds, which on a rate-limited
  endpoint can mean a handful of rows out of 271 with no error. **A scheduled
  refresh that writes to a database should use the strict variants.**
- `get_active_pools()` alongside `get_all_pools()`. Retired pools still resolve
  by name so no stored key breaks, but are excluded from bulk fetches.
- `PoolsDataError::is_retryable()` is now public.
- `--unnamed-min-sol` (default 10000): a pool no upstream source names is
  withheld rather than given a permanent `unnamed_*` name, unless it is large
  enough that missing it would be worse. It appears automatically once named or
  once larger.

### Fixed

Availability and correctness, most reachable in normal operation:

- **Remote denial of service.** A stake account's lockup timestamp was read as
  unsigned where the protocol defines it signed. Because a whole
  `getProgramAccounts` response decodes in one pass, a single account with a
  negative lockup failed *every* account for that pool. Anyone could create such
  an account naming a pool's authority, for about 0.0023 SOL, and break that
  pool permanently. Fixed, with a regression test over a real 221-account
  fixture.
- **Fragile deserialization.** `rentEpoch` and an inner `space` were required
  but never read; `rentEpoch` and `rentExemptReserve` are deprecated upstream.
  The day a node stops sending one, every pool would have failed at once. This
  already happened here once, with `warmupCooldownRate`, on 2026-08-03.
- **Silent data corruption.** Statistics accumulators were unchecked `u64` sums.
  A release build wraps, so an out-of-range value from a faulty endpoint
  returned a *successful* pool carrying a wrong total: measured, `u64::MAX + 10`
  came back as `9` with `Ok`. Values past the total SOL supply are now rejected
  at parse time, sums saturate, and a per-pool aggregate ceiling applies.
- **Partial refreshes looked successful.** See the strict variants above.
- **Redirects multiplied RPC cost.** The HTTP client followed up to 10
  redirects, so one pool's query could become 11 requests and a full refresh
  3,234. A redirect is now a configuration error, not followed and not retried.
- **Retries ignored the retry classification.** Permanent failures (invalid
  params, bad credentials, a misconfigured URL) spent the whole retry budget per
  pool. They now stop on the first attempt. 429 still backs off.
- **Retries bypassed the rate limit.** The limiter was consulted once before the
  retry loop, so retries were unpaced: three attempts at a configured 1 req/s
  completed in 7.5 ms. Every attempt now takes a permit.
- Reserve-only pools (holding everything in the reserve account, so zero stake
  accounts) are a success with zeroed data plus a warning, not an error. About
  81 discovered pools are in this state at any time.

### Changed

- **MSRV 1.75 → 1.82.** The declared 1.75 was already inaccurate: `icu_*` via
  `reqwest → url → idna` has required 1.82 since before this work. Established
  by compiling, not by reading declarations. The `discover` feature needs 1.89,
  which is why it is optional.
- `fetch_all_pools` now iterates active pools only; retired ones still resolve
  by name.
- Documentation on the lenient methods now states plainly that they return
  partial results.

### Compatibility

**No breaking change for existing consumers.** Verified against the live
production API rather than inferred:

| Check | Result |
| --- | --- |
| All 61 backend pool names resolve | pass |
| All 61 fetch successfully | pass, zero failures |
| Any name rebound to a different pool | none |
| Response schema | unchanged |
| Any existing pool returning empty | none |
| Entries removed | none; 210 pure additions |

Identity is the authority pubkey and names are frozen: a name, once emitted, is
never changed or deleted by the generator. That is why a run which renamed 65
pools and withheld 24 touched none of the 61 in production.

Public API additions are additive. No type, field or signature was removed or
renamed.

### Known issues, not fixed here

- **`active_stake_lamports` means two different things.**
  `fetch_all_pools_with_stats` (what the API serves) sums each account's full
  balance; `fetch_pools` sums delegated stake. Measured 213,469 SOL apart across
  271 pools, 0.158%. The gap is rent-exempt reserve plus undelegated balance.
  Decision pending: align the value, rename the field, or expose both. **This is
  the only pending change that would alter existing rows.**
- `activating_accounts` and `deactivated_accounts` are always 0 on
  `fetch_pools` / `fetch_pools_strict` results. That path has no epoch, so it
  cannot classify those states; it is documented in the code but the field names
  promise more than they deliver. Use `fetch_all_pools_with_stats*` for a true
  per-state breakdown.
- The stake-account query filters on `authorized.staker`, which anyone can set.
  Accounts a pool cannot manage are therefore attributed to it. Filtering on the
  withdrawer as well would fix it for SPL-family pools but silently zero the
  non-SPL hand-maintained ones: `marinade` has 166 accounts and none with
  `staker == withdrawer`. Documented rather than changed.
- Nothing bounds how many accounts a pool's query returns. Roughly 23 SOL of
  rent buys 10,000 planted accounts against one pool, and under a strict refresh
  one slow pool fails the whole run.

### Deploy checklist

1. Switch the refresh to `fetch_all_pools_strict` (or
   `fetch_all_pools_with_stats_strict`). Without it a rate-limited endpoint can
   write a fraction of the rows and report success.
2. Confirm the write path tolerates **210 new rows**, and that nothing joins on
   `authority_address`.
3. Note the RPC cost: one `getProgramAccounts` per pool, so 61 → 271 requests
   per refresh, about 4.4x. Bandwidth barely moves (+4 MB) because the new pools
   are small; it is request *count* that grows. At the public preset (1 req/s,
   concurrency 1) a full refresh takes roughly 5 minutes. Raise `--min-sol` to
   shrink the registry if that does not fit.
4. Four rows (`dynosol`, `definity`, `layer33`, `starpool`) store a pool address
   in `authority_address` where the library uses the withdraw authority. This
   predates this release. If the pipeline persists `authority`, those four rows
   get corrected on the next write.
5. Decide `active_stake_lamports` before deploying if any dashboard labels it as
   stake.
