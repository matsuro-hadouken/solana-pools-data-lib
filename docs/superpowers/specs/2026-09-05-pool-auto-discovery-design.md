# Pool Auto-Discovery and Naming

**Date:** 2026-09-05
**Status:** Approved. Audited over three rounds; see Audit Record.
**Plan:** `docs/superpowers/plans/2026-09-10-pool-auto-discovery.md`

## Problem

`src/pools.rs` holds a hand-written `Lazy<Vec<PoolInfo>>` of 61 `(name, authority)` pairs.
Adding a pool means finding its authority pubkey by hand and committing it. The registry is
the library's entire universe of pools: `fetch_pools` resolves names against it and
`fetch_all_pools` iterates it.

Measured against mainnet on 2026-09-05, 1,693 SPL-family stake pools exist. The registry
covers 28 of them. Pools with real deposits are being missed.

## Scope

**In scope:** SPL-family stake pools (SPL, Sanctum SPL, Sanctum SPL Multi). These are
enumerable on-chain and nameable from their LST mint.

**Out of scope:** custodial and institutional stakers (kraken, binance, figment, galaxy,
kiln, twinstake, p2p, okx, upbit) and non-SPL protocols (marinade, lido, socean, eversol,
foundation, firedancer_delegation). 33 registry entries. No on-chain name source exists for
these; they stay hand-maintained. A chain-wide census to *detect* them was explored and
deferred — see Appendix.

## Mechanism

### Discovery

A pool's `authority` in the registry is the SPL stake pool withdraw authority, a PDA over
`[pool_address, "withdraw"]`. Verified: `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`
derives to `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS`, matching `src/pools.rs:36`.

Three programs share an identical 611-byte `StakePool` layout:

| Program | ID | Live pools |
|---|---|---|
| SPL Stake Pool | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` | 232 |
| Sanctum SPL | `SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY` | 1401 |
| Sanctum SPL Multi | `SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn` | 56 |

One `getProgramAccounts` per program, filtered `dataSize: 611` plus `memcmp` at offset 0
matching `AccountType::StakePool` (byte `1`, base58 `"2"`).

Fields read from each account:

| Offset | Field | Use |
|---|---|---|
| 97 | `stake_withdraw_bump_seed` (u8) | PDA derivation |
| 162..194 | `pool_mint` (Pubkey) | naming key |
| 258..266 | `total_lamports` (u64 LE) | size threshold |
| 274..282 | `last_update_epoch` (u64 LE) | staleness guard |

The stored bump lets us skip the `find_program_address` bump-search loop, but **not** the
off-curve check:

```rust
let authority = Pubkey::create_program_address(
    &[pool.as_ref(), b"withdraw", &[bump]], &program_id,
)?;   // errors InvalidSeeds if the result is on-curve
```

Verified: reproduces all 28 currently-derivable registry entries exactly.

The off-curve check is not optional. If the bump is corrupt, offset 97 drifts, or a fork
changes the layout, a bare hash silently yields an on-curve pubkey that is not a valid PDA
and cannot own anything — a garbage authority written into the registry as though it were
real. `create_program_address` turns that silent corruption into a hard error.

`solana-pubkey` provides both the derivation and base58, so it *replaces* `sha2` and `bs58`
rather than adding to them.

### Naming

`pool_mint` is looked up in the Sanctum LST list
(`https://raw.githubusercontent.com/igneous-labs/sanctum-lst-list/master/sanctum-lst-list.toml`),
245 entries mapping mint to `symbol` and `name`.

**Use the `name` field, not `symbol`.** The registry's convention is *entity* names
(`binance_2`, `kraken`, `figment`, `jito`), not ticker symbols. Symbols produce `gtsol`,
`psol`, `dfdvsol`; the `name` field with its LST boilerplate stripped produces the entity:

| Sanctum `name` | slug | operator's own name |
|---|---|---|
| `Phantom Staked SOL` | `phantom` | phantom |
| `Sanctum Staked SOL` | `sanctum` | sanctum |
| `Gate Wrapped SOL` | `gate` | gate.io |
| `DeFi Development Corp Staked SOL` | `defi_development_corp` | defidevcorp |

Boilerplate strip: trailing `(liquid )?(staked|wrapped|restaked)? sol(ana)?`, case-insensitive.
Then lowercase and replace runs of non-alphanumerics with `_`.

The rule is imperfect (`The Vault` -> `the_vault`, `JPOOL Solana Token` ->
`jpool_solana_token`), but under the freeze rule those pools keep their existing names, so
imperfect slugs only ever land on genuinely new pools where a human is reviewing the diff.

**Guards, because the freeze rule makes a bad slug permanent.** The strip can eat a whole
legitimate brand — `Wrapped SOL`, `Liquid Solana`, or any operator whose brand genuinely
ends in "Sol". So: if the stripped result is empty or under 3 characters, discard it and
fall back to `unnamed_<pool8>` with a TODO. If stripping changed the string at all and the
remainder is a single short token, emit the name with a `// verify: from "<original>"`
comment so it gets a second look during diff review.

**Alias overrides.** A small table in the generator maps mint to an operator-preferred name,
for cases where the on-chain name is not what the operator calls the entity. Seeded from
ground truth: `GTSOL -> gate_io`, `dfdvSOL -> defidevcorp`. Aliases take precedence over the
slug rule but not over an existing frozen name.

**Name precedence, in order:**

1. **Existing registry name, matched by authority pubkey.** Frozen forever.
2. Sanctum `name`, boilerplate-stripped and slugified (see above).
3. `unnamed_<first 8 chars of pool address>`, emitted with a `// TODO: name` comment.

**Retention uses a third section, not a comment.** Trailing comments are invisible to code,
so retained-dead pools would silently accumulate into `fetch_all_pools()` and be reported
forever as successful empty pools — "all pools" would quietly come to mean "all pools ever
generated." Retired entries move to a `// ---- RETIRED ----` block instead, and:

- `get_all_pools()` keeps returning them, so `get_pool_by_name()` still resolves and no API
  key breaks;
- a new `get_active_pools()` excludes them, and `fetch_all_pools()` uses it.

A generated entry is never *deleted* once emitted, even if the pool later
falls below `--min-sol` or disappears from the program. Removal would delete a live API key,
which is the exact failure the freeze invariant exists to prevent. A pool that drops out is
retained with a `// below threshold as of epoch N` or `// no longer on-chain` comment; only
a human deletes entries.

**Migration.** If an LST moves to a new pool address, its withdraw authority changes, so it
appears as a new pool with a colliding slug and gets a `_2` suffix while the old entry is
retained-and-commented. The generator cannot detect that these are the same operator —
merging them is a human edit. *As shipped, no same-mint-different-authority warning is
emitted:* the generator keeps no mint index, so a migration surfaces only as the `_2` entry
appearing next to the retained original in the diff. What it does flag to stderr is the
narrower case of two pools deriving the **same** authority, where it keeps the first
deterministically.

Rule 1 is the invariant: **the authority pubkey is the pool's identity; the name is a
mutable label that the generator may add but never change.** Pool names are API keys, and a
regeneration that renames `forward_industries` to `dumsol` would silently break every
consumer keyed on the old name.

*Not shipped:* the design called for a trailing `// sanctum: dumSOL` comment wherever the
frozen name disagrees with the on-chain symbol. The generator emits no such note — a frozen
entry keeps whatever note it already carried, plus the generator-owned markers
(`stale: ...`, `below threshold ...`). Divergences are found by reading the Sanctum list,
not from `src/pools.rs`.

Name collisions between two different authorities are resolved by appending `_2`, `_3`,
matching the existing registry convention. Measured: 2 collisions today (`binance` against
the existing manual `binance`; `sanctum` against the existing `sanctum`, needing `sanctum_3`).

**Suffix assignment must be deterministic.** When several *new* pools slug to the same base
in one run, they are sorted by authority pubkey before suffixes are handed out. Otherwise
`sanctum_3` and `sanctum_4` could swap between runs, which would violate the freeze
invariant for exactly the pools the invariant exists to protect. After the first run the
assignment is read back out of `pools.rs` and frozen like any other name.

### Threshold

`--min-sol` (default `1`) filters on `total_lamports`. Measured distribution:

| min SOL | pools | auto-named | unnamed |
|---|---|---|---|
| 0 | 1693 | 240 | 1453 |
| **1** | **261** | **173** | **88** |
| 10 | 180 | 138 | 42 |
| 100 | 128 | 104 | 24 |
| 1000 | 80 | 68 | 12 |

**`total_lamports` is a cached field, not a live balance.** It is only refreshed by
`UpdateStakePoolBalance`, once per epoch. Measured at epoch 1028: of the 261 pools over the
1 SOL threshold, 217 were current, but **41 were more than 10 epochs stale**. For an
abandoned pool the field reports its balance at the moment it stopped being maintained, so
a pool that has since drained to zero still passes the threshold, and one that has grown
may fail it.

Mitigation, asymmetric by direction of harm:

- **Already-known pools** keep a soft `// stale: last updated epoch 812` comment. A hard
  filter would evict live pools that simply have not been cranked, which is worse.
- **Brand-new entries** may not be admitted on stale data alone. If a pool is >10 epochs
  behind and its cached `total_lamports` is what puts it over the threshold, the generator
  sums the live lamports of its stake accounts and uses that instead. Otherwise a pool that
  drained to zero years ago is admitted on a fossil balance.

Chosen default: **1 SOL, giving 261 pools.** Accepted consequences:

- `fetch_all_pools()` issues 261 `getProgramAccounts` calls instead of 61, roughly 52
  seconds at `rate_limit(5)`. Retunable via `--min-sol` without code changes.
- 88 pools ship with placeholder names.
- 81 pools hold under 10 SOL and will frequently have zero stake accounts.

## Components

### `examples/discover_pools.rs` (new)

Placed under `examples/` so its dependencies land in `[dev-dependencies]` and never enter
the dependency tree of anything consuming this library. The directory already holds 8
examples; no new structure is introduced.

```
cargo run --example discover_pools -- --min-sol 1
git diff                      # human reviews names
```

Writes `src/pools.rs` in place. The path is hardcoded — there is no `--out` flag, and the
generator rejects every unrecognized argument by name, so passing one is a hard error rather
than a silent no-op. Its only flags are `--min-sol <SOL>` and `--verify`. It must **not** use
a stdout redirect: `> src/pools.rs` truncates the file at redirect time, before the process
starts, destroying the MANUAL block and the existing name bindings the generator needs to
read. Output goes to a sibling temp file and is renamed over the target only after a
successful run.

Steps: fetch 3 programs -> parse fields -> derive authority -> filter by threshold ->
fetch Sanctum list -> apply name precedence -> emit `src/pools.rs`.

Reads the current `src/pools.rs` first, to recover existing authority-to-name bindings and
the MANUAL block, then rewrites it.

**Dependencies, MSRV-gated.** `Cargo.toml:13` declares `rust-version = "1.75"`, but every
published `solana-pubkey` exceeds it (3.0.0 needs 1.81; 4.x needs 1.89), and host-side
`create_program_address` sits behind the `curve25519` feature. A plain dev-dependency would
break `cargo test` for anyone on 1.75.

So they become *optional* dependencies behind a feature, and the example requires it:

```toml
[features]
discover = ["dep:solana-pubkey", "dep:toml"]

[dependencies]
solana-pubkey = { version = "3.0", features = ["curve25519"], optional = true }
toml          = { version = "0.8", optional = true }

[[example]]
name = "discover_pools"
required-features = ["discover"]
```

```
cargo run --features discover --example discover_pools -- --min-sol 1
```

The library's 1.75 promise is unchanged for consumers and for CI: an inactive feature's
optional dependencies are resolved into `Cargo.lock` but never compiled, so their MSRV is
not enforced as a build unit. Only the maintainer running the generator needs 1.81+.

Two caveats. `cargo --all-features` activates `discover` and will therefore require 1.81+ by
design; CI must not use it for the MSRV job. And if `Cargo.lock` is ever regenerated by a
much newer Cargo, the lockfile *format version* can break 1.75 before compilation is even
reached — that is independent of this feature, but it is the thing most likely to falsify
the promise in practice.

### `src/pools.rs` (regenerated, structure changed)

Gains three section markers so hand-maintained entries survive regeneration and retired
pools stay resolvable without polluting `fetch_all_pools()`:

```rust
// ---- MANUAL: custodial and non-SPL pools. Edit freely. ----
PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"),
// ---- END MANUAL ----

// ---- GENERATED by `cargo run --example discover_pools`. Do not edit. ----
PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
// ---- END GENERATED ----

// ---- RETIRED: kept so existing API names still resolve. Excluded from
//      get_active_pools() and therefore from fetch_all_pools(). ----
PoolInfo::new("socean", "AzZRvyyMHBm8EHEksWxq4ozFL7JxLMydCDMGhqM6BVck"),
// ---- END RETIRED ----
```

The generator rewrites all three blocks. MANUAL is not passed through verbatim: its entries
are re-parsed and re-emitted from the same renderer as GENERATED, which sorts by authority
and normalizes spacing. What survives is the content — each entry's name, authority and
trailing `// note` — not the byte layout, and not any non-entry line, which the parser now
rejects outright rather than dropping. Only text *outside* the markers is byte-preserved,
which is why the "do not edit" guidance in `src/pools.rs` sits above `POOLS_ACTIVE` rather
than inside a block. The public API of the module is unchanged.

**First-run bootstrap.** The committed `src/pools.rs` has no markers today, so "abort when
markers are missing" would make the first run impossible. When no markers are found, the
generator performs a one-time migration instead: parse the flat `PoolInfo::new` list, and
split it by whether each authority appears in the derived set — into MANUAL
(33 entries: kraken, figment, marinade, lido, …) and GENERATED (28 entries).
**The derived set used for classification is pre-threshold** (all 1,693 pools, ignoring
`--min-sol`). Using the post-threshold set would misclassify a known SPL pool that happens
to sit below the current threshold as MANUAL, and it would then never be regenerated. It prints the
classification to stderr for review. This path runs once; afterwards markers exist and a
missing marker is a genuine error.

### `src/client.rs` (one targeted fix)

`client.rs:317` returns `PoolsDataError::NoStakeAccounts` when a pool has zero stake
accounts. With 81 sub-10-SOL pools in the registry this fires routinely on healthy pools
that hold their balance in reserve, turning normal operation into reported failures.

Change: an empty account list produces a successful `PoolData` with an empty
`stake_accounts` vec, zeroed statistics, and an empty `validator_distribution`.
`NoStakeAccounts` is removed from the error path.

This is in scope because auto-discovery is what makes the case common.

**But silence here would hide a bad authority.** An empty result is indistinguishable from a
mis-derived authority, a stale manual entry, or a migrated pool. So emptiness must stay
visible rather than becoming invisible:

- the empty case logs at `warn!` with the pool name and authority;
- `--verify` on the generator queries each newly-derived authority, moving the check to
  generation time where a human is present. It must not treat one empty response as proof
  of a bad authority: a legitimate pool can hold everything in reserve, and RPC returns
  transient empties. So a zero result is **retried once at a later slot**, and only a
  result that is still empty counts.

  A still-empty *new* authority is **withheld from the output**, not marked and emitted.
  The two outcomes are not symmetric. Withholding a real pool costs one later run — it is
  admitted as soon as it holds stake. Emitting a mis-derived authority freezes a wrong name
  onto a public API key permanently. Every withheld pool is reported to stderr by name and
  authority so the operator can resolve it.

  Marking is for entries that **already exist** in the registry and have merely gone quiet.
  Those keep their names — a live API key is never withdrawn — and carry a `verify: no
  stake accounts` note for a human to look at.

  Above both sits the signal that actually means the derivation broke, and that check is
  **per program, not global**. A global "all new authorities empty" abort never fires when
  only one of the three programs diverges: the other two return healthy results, and the
  diverged program's wrong authorities go out with them. So each program's cohort is
  evaluated on its own, and a program whose entire new cohort verifies empty aborts the
  run outright.

### `src/pools.rs` invariants (new tests)

`POOLS_BY_AUTHORITY` (`pools.rs:105`) is built with `collect()`, so two entries sharing an
authority silently overwrite each other and misattribute fetched stake accounts. Existing
tests check duplicate *names* only. Add a duplicate-*authority* test covering the MANUAL, GENERATED, and RETIRED blocks
together, and have the generator abort on a cross-block collision. A retired entry sharing
an authority with a live one is the likeliest such collision, since retirement is how
migrated pools are parked.

### Determinism and provenance

The GENERATED block is sorted by authority pubkey so regeneration produces a minimal diff.
Output is rustfmt-compatible and idempotent: running the generator twice against an
unchanged chain state produces a byte-identical file.

A header comment records the RPC endpoint, slot, epoch, `--min-sol`, and the Sanctum list
revision, so a surprising diff can be traced to what changed. The endpoint is recorded as
**scheme and host only**: this line is committed, and a Helius or Alchemy URL carries its
API key in the path or query string, so recording it verbatim would publish a credential
to git history. The counts quoted in this spec
were measured at epoch 1028 against `api.mainnet-beta.solana.com`.

## Data Flow

```
3x getProgramAccounts ─┐
                       ├─> [(authority, pool, mint, total_lamports)]
   read bump@97 ───────┘              │
                                      │ filter total_lamports >= min_sol
                                      v
   Sanctum LST list ────────> apply name precedence ──> emit src/pools.rs
   existing pools.rs ───────/                                  │
                                                               v
                                                     human reviews git diff
```

Runtime is untouched: the library still reads a static embedded registry with zero RPC
cost at request time.

## Error Handling

The generator is an operator tool run by hand; it fails loudly and writes nothing partial.

- Any of the 3 `getProgramAccounts` calls failing: abort non-zero. A partial pool set would
  silently delete pools from the registry.
- Sanctum list unreachable: abort non-zero rather than regenerating with every name
  degraded to `unnamed_*`, which would look like mass renaming in the diff.
- Any account whose `data.len() != 611`, or whose `pool_mint` is all zeros: skip, warn to
  stderr, continue.
- A program returning zero `dataSize: 611` accounts: abort. That means the layout or program
  ID assumption has broken, not that every pool vanished.
- Existing `src/pools.rs` unparseable or missing a section marker: abort non-zero. Losing
  the manual block loses 33 hand-curated pools.

`src/pools.rs` is read fully into memory before anything is written, and the new content
lands in a temp file that is renamed over the target only on success. An aborted run leaves
the committed registry untouched.

## Testing

Generator, in `examples/discover_pools.rs` under `#[cfg(test)]`:

- Authority derivation against a hardcoded fixture: Jito's pool address and bump must
  produce `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS`. This is the one piece of
  non-trivial logic and the check that fails if the layout or hash construction breaks.
- Name precedence: an authority already present keeps its name even when the Sanctum
  `name` differs (regression guard for `forward_industries` vs `Dum Staked SOL`).
- Slug rule: `Phantom Staked SOL` -> `phantom`; `Gate Wrapped SOL` -> `gate` (then the alias
  table maps it to `gate_io`); a name that strips to empty or <3 chars falls back to
  `unnamed_*` rather than emitting a mangled slug.
- Collision suffixing produces `_2`/`_3` deterministically, ordered by authority pubkey.
- Duplicate authority across MANUAL, GENERATED, and RETIRED blocks aborts.

Library:

- Existing `src/pools.rs` tests continue to pass unchanged (unique names, valid base58
  authorities, known lookups).
- New test in `client.rs`: a pool returning zero stake accounts yields `Ok` with empty
  `stake_accounts`, not `Err(NoStakeAccounts)`.

No network calls in tests. The generator's RPC layer is exercised by running it.

## Appendix: deferred census

Detecting non-SPL stakers is possible but was scoped out. `memcmp` accepts a partial byte
match, so filtering on the first byte of `authorized.staker` at offset 12 partitions all
~1.4M stake accounts into 256 buckets. One measured bucket returned 4,880 accounts in 2.1 MB
in 2.9 seconds on the public RPC, implying a full census costs roughly 13 minutes and 500 MB.

Aggregating that bucket by staker surfaced an untracked authority holding 566,429 active SOL.
Extrapolated chain-wide, about 1,000 authorities hold 100k+ SOL. Pool authorities are
separable from custodial ones by validator spread: pools fan out across many validators,
custodial positions concentrate on one or two.

The blocker is naming, not detection: this channel produces a review queue of unlabelled
pubkeys, not names. Revisit if the manual list becomes a burden.

## Audit Record

Reviewed in three adversarial rounds. Defects found and closed:

| # | Defect | Resolution |
|---|---|---|
| 1 | First run impossible — spec aborted on missing section markers, but `pools.rs` has none | One-time bootstrap classifying the flat list against the pre-threshold derived set |
| 2 | Bare-sha256 PDA skipped the off-curve check, so a bad bump emits a garbage authority | `create_program_address`, which errors `InvalidSeeds` |
| 3 | `solana-pubkey` MSRV (1.81+) exceeds the repo's declared 1.75 | Optional dependency behind a `discover` feature; `cargo test` at 1.75 unaffected |
| 4 | `--verify` aborted only if *all* new authorities were empty, so one diverged program still emitted wrong authorities | Per-program cohort abort; marked-empty new entries withheld |
| 5 | `POOLS_BY_AUTHORITY` silently overwrites duplicates; tests covered duplicate names only | Duplicate-authority test across all three blocks; generator aborts on collision |
| 6 | Retention by trailing comment is invisible to code — dead pools would accumulate into `fetch_all_pools()` | `RETIRED` section plus `get_active_pools()` |
| 7 | `total_lamports` is cached, not live; 41/261 pools >10 epochs stale | Asymmetric: soft comment for known pools, live recomputation before admitting a new stale one |
| 8 | Naming from ticker symbol (`gtsol`) violated the registry's entity convention (`gate`) | Sanctum `name` field with boilerplate stripped, plus alias overrides |
| 9 | Boilerplate strip can eat a legitimate brand, and the freeze rule makes it permanent | Empty/<3-char strips fall back to `unnamed_*`; suspicious strips flagged for review |
| 10 | Collision suffixes could swap between runs, breaking the freeze invariant | New colliding pools sorted by authority pubkey before suffixing |

Verified against mainnet at epoch 1028, not inferred: `pool_mint`@162 located by byte-search
of Jito's live account; bump@97 derivation reproduces all 28 currently-derivable registry
authorities; six previously-untracked pools (sctmSOL, PSOL, dfdvSOL, GTSOL and two unnamed,
~6.5M SOL combined) confirmed to return stake accounts through the library's exact
`memcmp @12` query.
