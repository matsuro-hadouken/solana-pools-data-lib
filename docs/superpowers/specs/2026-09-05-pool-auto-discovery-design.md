# Pool Auto-Discovery and Naming

**Date:** 2026-09-05
**Status:** Approved, not yet implemented

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

An earlier draft computed this as a bare `sha256(pool || "withdraw" || bump || program_id ||
"ProgramDerivedAddress")` to avoid a dependency. That is wrong at a trust boundary. If the
bump is corrupt, offset 97 drifts, or a fork changes the layout, a bare hash silently emits
an on-curve pubkey that is not a valid PDA and cannot own anything — a garbage authority
written into the registry as though it were real. `create_program_address` costs one dev-
dependency and turns that silent corruption into a hard error.

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
imperfect slugs only ever land on genuinely new pools where a human is reviewing the diff
anyway.

**Name precedence, in order:**

1. **Existing registry name, matched by authority pubkey.** Frozen forever.
2. Sanctum `name`, boilerplate-stripped and slugified (see above).
3. `unnamed_<first 8 chars of pool address>`, emitted with a `// TODO: name` comment.

**Retention.** A generated entry is never removed once emitted, even if the pool later
falls below `--min-sol` or disappears from the program. Removal would delete a live API key,
which is the exact failure the freeze invariant exists to prevent. A pool that drops out is
retained with a `// below threshold as of epoch N` or `// no longer on-chain` comment; only
a human deletes entries.

**Migration.** If an LST moves to a new pool address, its withdraw authority changes, so it
appears as a new pool with a colliding slug and gets a `_2` suffix while the old entry is
retained-and-commented. The generator cannot detect that these are the same operator —
merging them is a human edit. Same-mint-different-authority is flagged to stderr so the
operator sees the case rather than discovering it later.

Rule 1 is the invariant: **the authority pubkey is the pool's identity; the name is a
mutable label that the generator may add but never change.** Pool names are API keys, and a
regeneration that renames `forward_industries` to `dumsol` would silently break every
consumer keyed on the old name.

Where the frozen name disagrees with the on-chain symbol, the generator appends a trailing
comment (`// sanctum: dumSOL`) so divergences are visible and can be resolved by a
deliberate human edit.

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

Mitigation: read `last_update_epoch`@274 alongside it and emit a trailing comment
(`// stale: last updated epoch 812`) on any pool more than 10 epochs behind, so the
operator sees which threshold decisions rest on stale data. Not a hard filter — a stale
pool can still hold real stake, and the library reads live stake accounts at fetch time
regardless. The threshold is a registry-inclusion heuristic, not a balance report.

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

Writes `src/pools.rs` in place via `--out` (default `src/pools.rs`). It must **not** use a
stdout redirect: `> src/pools.rs` truncates the file at redirect time, before the process
starts, destroying the MANUAL block and the existing name bindings the generator needs to
read. Output goes to a sibling temp file and is renamed over the target only after a
successful run.

Steps: fetch 3 programs -> parse fields -> derive authority -> filter by threshold ->
fetch Sanctum list -> apply name precedence -> emit `src/pools.rs`.

Reads the current `src/pools.rs` first, to recover existing authority-to-name bindings and
the MANUAL block, then rewrites it.

New dev-dependencies: `solana-pubkey`, `toml`. Dev-only, so nothing reaches consumers.

### `src/pools.rs` (regenerated, structure changed)

Gains two section markers so hand-maintained entries survive regeneration:

```rust
// ---- MANUAL: custodial and non-SPL pools. Edit freely. ----
PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"),
// ---- END MANUAL ----

// ---- GENERATED by `cargo run --example discover_pools`. Do not edit. ----
PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
// ---- END GENERATED ----
```

The generator copies the MANUAL block through verbatim and rewrites only the GENERATED
block. The public API of the module is unchanged.

**First-run bootstrap.** The committed `src/pools.rs` has no markers today, so "abort when
markers are missing" would make the first run impossible. When no markers are found, the
generator performs a one-time migration instead: parse the flat `PoolInfo::new` list, and
split it by whether each authority appears in the freshly derived set — into MANUAL
(33 entries: kraken, figment, marinade, lido, …) and GENERATED (28 entries). It prints the
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
- `--verify` on the generator queries each newly-derived authority once and refuses to emit
  any that returns zero stake accounts, moving the check to generation time where a human
  is present, instead of leaving it to silently degrade a production fetch.

### `src/pools.rs` invariants (new tests)

`POOLS_BY_AUTHORITY` (`pools.rs:105`) is built with `collect()`, so two entries sharing an
authority silently overwrite each other and misattribute fetched stake accounts. Existing
tests check duplicate *names* only. Add a duplicate-*authority* test covering the MANUAL and
GENERATED blocks together, and have the generator abort on a cross-block collision.

### Determinism and provenance

The GENERATED block is sorted by authority pubkey so regeneration produces a minimal diff.
Output is rustfmt-compatible and idempotent: running the generator twice against an
unchanged chain state produces a byte-identical file.

A header comment records the RPC endpoint, slot, epoch, `--min-sol`, and the Sanctum list
commit, so a surprising diff can be traced to what changed. The counts quoted in this spec
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
- An account shorter than 266 bytes, or a `pool_mint` of all zeros: skip that pool, warn to
  stderr, continue.
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
  symbol differs.
- Collision suffixing produces `_2` on a duplicate symbol.

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
