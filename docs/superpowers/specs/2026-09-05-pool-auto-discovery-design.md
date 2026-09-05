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

Because the bump is stored on-chain, the authority is a single hash — no
`find_program_address` bump-search loop and no ed25519 curve check:

```
authority = base58(sha256(pool || "withdraw" || [bump] || program_id || "ProgramDerivedAddress"))
```

Verified: reproduces all 28 currently-derivable registry entries exactly.

### Naming

`pool_mint` is looked up in the Sanctum LST list
(`https://raw.githubusercontent.com/igneous-labs/sanctum-lst-list/master/sanctum-lst-list.toml`),
245 entries mapping mint to `symbol` and `name`.

**Name precedence, in order:**

1. **Existing registry name, matched by authority pubkey.** Frozen forever.
2. Sanctum `symbol`, lowercased with every non-alphanumeric character replaced by `_`
   (`JitoSOL` -> `jitosol`, `dumSOL` -> `dumsol`). Not word-split: `JitoSOL` does **not**
   become `jito_sol`.
3. `unnamed_<first 8 chars of pool address>`, emitted with a `// TODO: name` comment.

Rule 1 is the invariant: **the authority pubkey is the pool's identity; the name is a
mutable label that the generator may add but never change.** Pool names are API keys, and a
regeneration that renames `forward_industries` to `dumsol` would silently break every
consumer keyed on the old name.

Where the frozen name disagrees with the on-chain symbol, the generator appends a trailing
comment (`// sanctum: dumSOL`) so divergences are visible and can be resolved by a
deliberate human edit.

Name collisions between two different authorities are resolved by appending `_2`, `_3`,
matching the existing registry convention.

### Threshold

`--min-sol` (default `1`) filters on `total_lamports`. Measured distribution:

| min SOL | pools | auto-named | unnamed |
|---|---|---|---|
| 0 | 1693 | 240 | 1453 |
| **1** | **261** | **173** | **88** |
| 10 | 180 | 138 | 42 |
| 100 | 128 | 104 | 24 |
| 1000 | 80 | 68 | 12 |

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

New dev-dependencies: `sha2`, `bs58`, `toml`.

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

### `src/client.rs` (one targeted fix)

`client.rs:317` returns `PoolsDataError::NoStakeAccounts` when a pool has zero stake
accounts. With 81 sub-10-SOL pools in the registry this fires routinely on healthy pools
that hold their balance in reserve, turning normal operation into reported failures.

Change: an empty account list produces a successful `PoolData` with an empty
`stake_accounts` vec, zeroed statistics, and an empty `validator_distribution`.
`NoStakeAccounts` is removed from the error path.

This is in scope because auto-discovery is what makes the case common.

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
