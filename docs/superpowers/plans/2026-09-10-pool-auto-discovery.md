# Pool Auto-Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace hand-maintained pool discovery with an offline generator that enumerates SPL-family stake pools, derives their withdraw authorities, names them from their LST mint, and regenerates `src/pools.rs` for human review.

**Architecture:** Pure logic lives in `src/discovery.rs` behind a `discover` feature so it is unit-testable and never compiled for consumers. `examples/discover_pools.rs` is a thin main that does RPC and wires the pieces together. The library's runtime is unchanged except for a retired-pool split and empty-pool tolerance.

**Tech Stack:** Rust 2021, reqwest + tokio (already present), `solana-pubkey` and `toml` as optional feature-gated dependencies.

**Spec:** `docs/superpowers/specs/2026-09-05-pool-auto-discovery-design.md`

## Global Constraints

- Library MSRV is `rust-version = "1.75"` (`Cargo.toml:13`). No change. Generator-only deps are optional and feature-gated; only a maintainer running the generator needs 1.81+.
- No new **default** dependencies. `solana-pubkey` and `toml` are `optional = true`, activated only by the `discover` feature.
- **Pool names are API keys.** A name, once emitted, is never changed or deleted by the generator. Identity is the authority pubkey.
- Generator output is deterministic: sorted by authority pubkey, rustfmt-clean, byte-identical across runs against unchanged chain state.
- Three programs, identical 611-byte `StakePool` layout:
  - SPL Stake Pool `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`
  - Sanctum SPL `SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY`
  - Sanctum SPL Multi `SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn`
- Field offsets: `bump@97`, `pool_mint@162..194`, `total_lamports@258..266`, `last_update_epoch@274..282`.
- Known-good fixture (mainnet, verified): pool `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`, bump `253`, mint `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`, authority `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS`.

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` | feature + optional dep wiring |
| `src/discovery.rs` (new) | all pure logic: account decode, authority derivation, slug rule, registry parse/emit, name assignment. Feature-gated, fully unit-tested. |
| `examples/discover_pools.rs` (new) | thin main: CLI args, RPC, per-program cohort verify, provenance header |
| `src/pools.rs` | gains RETIRED split, `get_active_pools()`, and the three section markers (Task 6, by hand); GENERATED body thereafter written by the generator |
| `src/client.rs` | empty stake-account list becomes success, not error |
| `src/lib.rs` | feature-gated `mod discovery` |

---

### Task 1: Feature gating and dependency wiring

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs:32-42`

**Interfaces:**
- Consumes: nothing
- Produces: `discover` feature; `solana_pubkey::Pubkey` and `toml` available under that feature; `crate::discovery` module path

- [ ] **Step 1: Add the feature and optional dependencies**

In `Cargo.toml`, replace the `[features]` block and add to `[dependencies]`:

```toml
[features]
default = []
discover = ["dep:solana-pubkey", "dep:toml"]
```

```toml
# generator-only; never built unless `discover` is enabled
solana-pubkey = { version = "3.0", features = ["curve25519"], optional = true }
toml          = { version = "0.8", optional = true }
```

Add the example target so it is skipped without the feature, and so `cargo test` runs its unit tests when the feature is on:

```toml
[[example]]
name = "discover_pools"
required-features = ["discover"]
```

- [ ] **Step 2: Declare the module behind the feature**

In `src/lib.rs`, alongside the existing `mod` declarations:

```rust
#[cfg(feature = "discover")]
pub mod discovery;
```

- [ ] **Step 3: Create the module stub so the crate compiles**

```bash
echo '//! Offline pool discovery. Compiled only with the `discover` feature.' > src/discovery.rs
```

- [ ] **Step 4: Verify the 1.75 promise is intact and the feature builds**

```bash
cargo build                      # default features: solana-pubkey must NOT appear
cargo tree -e normal | grep -c solana-pubkey || echo "absent as expected"
cargo build --features discover  # pulls solana-pubkey + toml
```

Expected: first build succeeds without `solana-pubkey` in the tree; second build succeeds.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/discovery.rs
git commit -m "build: add feature-gated discover deps"
```

---

### Task 2: StakePool account decoding and authority derivation

**Files:**
- Modify: `src/discovery.rs`

**Interfaces:**
- Consumes: `discover` feature from Task 1
- Produces:
  - `pub struct StakePool { pub pool: Pubkey, pub bump: u8, pub mint: Pubkey, pub total_lamports: u64, pub last_update_epoch: u64 }`
  - `pub fn decode(pool: Pubkey, data: &[u8]) -> Result<StakePool, DecodeError>`
  - `pub fn derive_authority(pool: &Pubkey, bump: u8, program: &Pubkey) -> Result<Pubkey, DecodeError>`
  - `pub enum DecodeError { BadLength(usize), NotStakePool(u8), ZeroMint, OnCurve }`

- [ ] **Step 1: Write the failing tests**

Append to `src/discovery.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const JITO_POOL: &str = "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb";
    const JITO_AUTH: &str = "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS";
    const JITO_MINT: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";
    const SPL_PROGRAM: &str = "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy";

    /// Minimal well-formed 611-byte StakePool account.
    fn fixture() -> Vec<u8> {
        let mut d = vec![0u8; 611];
        d[0] = 1;                       // AccountType::StakePool
        d[97] = 253;                    // Jito's real bump
        d[162..194].copy_from_slice(Pubkey::from_str(JITO_MINT).unwrap().as_ref());
        d[258..266].copy_from_slice(&10_264_121_881_528_565u64.to_le_bytes());
        d[274..282].copy_from_slice(&1031u64.to_le_bytes());
        d
    }

    #[test]
    fn decodes_documented_offsets() {
        let pool = Pubkey::from_str(JITO_POOL).unwrap();
        let sp = decode(pool, &fixture()).unwrap();
        assert_eq!(sp.bump, 253);
        assert_eq!(sp.mint, Pubkey::from_str(JITO_MINT).unwrap());
        assert_eq!(sp.total_lamports, 10_264_121_881_528_565);
        assert_eq!(sp.last_update_epoch, 1031);
    }

    #[test]
    fn rejects_wrong_length() {
        // Must be exactly 611. A 266-byte check would not cover last_update_epoch@274.
        let pool = Pubkey::from_str(JITO_POOL).unwrap();
        assert!(matches!(decode(pool, &vec![0u8; 610]), Err(DecodeError::BadLength(610))));
        assert!(matches!(decode(pool, &vec![0u8; 612]), Err(DecodeError::BadLength(612))));
    }

    #[test]
    fn rejects_non_stakepool_and_zero_mint() {
        let pool = Pubkey::from_str(JITO_POOL).unwrap();
        let mut d = fixture();
        d[0] = 2;                       // ValidatorList
        assert!(matches!(decode(pool, &d), Err(DecodeError::NotStakePool(2))));

        let mut d = fixture();
        d[162..194].fill(0);
        assert!(matches!(decode(pool, &d), Err(DecodeError::ZeroMint)));
    }

    #[test]
    fn derives_jito_authority() {
        let pool = Pubkey::from_str(JITO_POOL).unwrap();
        let prog = Pubkey::from_str(SPL_PROGRAM).unwrap();
        assert_eq!(
            derive_authority(&pool, 253, &prog).unwrap(),
            Pubkey::from_str(JITO_AUTH).unwrap()
        );
    }

    #[test]
    fn rejects_on_curve_derivation() {
        // A wrong bump usually lands on-curve; that must be an error, never a
        // silently-emitted garbage authority.
        let pool = Pubkey::from_str(JITO_POOL).unwrap();
        let prog = Pubkey::from_str(SPL_PROGRAM).unwrap();
        let failures = (0u8..=252)
            .filter(|b| derive_authority(&pool, *b, &prog).is_err())
            .count();
        // Deterministic: fixed pool, fixed program, fixed bump range. Measured 118.
        // The other 135 wrong bumps return a pubkey — create_program_address only
        // rejects on-curve results, it does not verify canonicity.
        assert_eq!(failures, 118, "on-curve rejection rate changed; off-curve check may be disabled");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --features discover discovery`
Expected: FAIL — `cannot find function decode`, `cannot find type StakePool`

- [ ] **Step 3: Write the implementation**

Put above the test module in `src/discovery.rs`:

```rust
use solana_pubkey::Pubkey;

pub const SPL_STAKE_POOL: &str = "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy";
pub const SANCTUM_SPL: &str = "SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY";
pub const SANCTUM_SPL_MULTI: &str = "SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn";

/// Every StakePool account is exactly this size across all three programs.
pub const STAKE_POOL_LEN: usize = 611;

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    BadLength(usize),
    NotStakePool(u8),
    ZeroMint,
    OnCurve,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLength(n) => write!(f, "expected {STAKE_POOL_LEN} bytes, got {n}"),
            Self::NotStakePool(t) => write!(f, "account type {t} is not StakePool"),
            Self::ZeroMint => write!(f, "pool_mint is all zeros"),
            Self::OnCurve => write!(f, "derived authority is on-curve, not a valid PDA"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakePool {
    pub pool: Pubkey,
    pub bump: u8,
    pub mint: Pubkey,
    pub total_lamports: u64,
    pub last_update_epoch: u64,
}

fn pubkey_at(data: &[u8], off: usize) -> Pubkey {
    let mut b = [0u8; 32];
    b.copy_from_slice(&data[off..off + 32]);
    Pubkey::new_from_array(b)
}

fn u64_at(data: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(b)
}

pub fn decode(pool: Pubkey, data: &[u8]) -> Result<StakePool, DecodeError> {
    if data.len() != STAKE_POOL_LEN {
        return Err(DecodeError::BadLength(data.len()));
    }
    if data[0] != 1 {
        return Err(DecodeError::NotStakePool(data[0]));
    }
    let mint = pubkey_at(data, 162);
    if mint == Pubkey::default() {
        return Err(DecodeError::ZeroMint);
    }
    Ok(StakePool {
        pool,
        bump: data[97],
        mint,
        total_lamports: u64_at(data, 258),
        last_update_epoch: u64_at(data, 274),
    })
}

/// The stored bump lets us skip find_program_address's search loop, but the
/// off-curve check still runs: create_program_address errors on an on-curve
/// result rather than returning a pubkey that cannot own anything.
pub fn derive_authority(
    pool: &Pubkey,
    bump: u8,
    program: &Pubkey,
) -> Result<Pubkey, DecodeError> {
    Pubkey::create_program_address(&[pool.as_ref(), b"withdraw", &[bump]], program)
        .map_err(|_| DecodeError::OnCurve)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --features discover discovery`
Expected: PASS, 5 tests

- [ ] **Step 5: Commit**

```bash
git add src/discovery.rs
git commit -m "feat(discovery): decode StakePool accounts and derive withdraw authority"
```

---

### Task 3: Slug rule, guards, and alias overrides

**Files:**
- Modify: `src/discovery.rs`

**Interfaces:**
- Consumes: nothing from prior tasks
- Produces:
  - `pub fn slugify(sanctum_name: &str) -> Slug`
  - `pub enum Slug { Clean(String), Suspicious { slug: String, original: String }, Unusable }`
  - `pub fn alias_for(symbol: &str) -> Option<&'static str>`

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/discovery.rs`:

```rust
#[test]
fn slugs_entity_names_not_tickers() {
    // The registry convention is entity names (binance_2, kraken), not tickers.
    assert_eq!(slugify("Phantom Staked SOL"), Slug::Clean("phantom".into()));
    assert_eq!(slugify("Sanctum Staked SOL"), Slug::Clean("sanctum".into()));
    assert_eq!(slugify("Jito Staked SOL"), Slug::Clean("jito".into()));
    assert_eq!(
        slugify("DeFi Development Corp Staked SOL"),
        Slug::Clean("defi_development_corp".into())
    );
}

#[test]
fn slugs_names_without_boilerplate() {
    assert_eq!(slugify("The Vault"), Slug::Clean("the_vault".into()));
    assert_eq!(slugify("JPOOL Solana Token"), Slug::Clean("jpool_solana_token".into()));
}

#[test]
fn refuses_to_eat_a_legitimate_brand() {
    // Stripping must not consume the whole name. The freeze rule would make
    // a mangled slug permanent API surface.
    assert_eq!(slugify("Wrapped SOL"), Slug::Unusable);
    assert_eq!(slugify("SOL"), Slug::Unusable);
    assert_eq!(slugify("Staked SOL"), Slug::Unusable);
}

#[test]
fn flags_short_strips_for_review() {
    // "Gate Wrapped SOL" -> "gate": real, but short enough to double-check.
    assert_eq!(
        slugify("Gate Wrapped SOL"),
        Slug::Suspicious { slug: "gate".into(), original: "Gate Wrapped SOL".into() }
    );
}

#[test]
fn aliases_beat_the_slug_rule() {
    assert_eq!(alias_for("GTSOL"), Some("gate_io"));
    assert_eq!(alias_for("dfdvSOL"), Some("defidevcorp"));
    assert_eq!(alias_for("JitoSOL"), None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --features discover slug`
Expected: FAIL — `cannot find function slugify`

- [ ] **Step 3: Write the implementation**

Add to `src/discovery.rs`. No regex crate: a suffix-strip loop is smaller than a dependency.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slug {
    /// Safe to emit as-is.
    Clean(String),
    /// Emit, but annotate for human review — the strip left very little behind.
    Suspicious { slug: String, original: String },
    /// Stripping consumed the name. Caller must fall back to `unnamed_*`.
    Unusable,
}

/// Operator ground truth where the on-chain name is not what the entity is called.
const ALIASES: &[(&str, &str)] = &[
    ("GTSOL", "gate_io"),
    ("dfdvSOL", "defidevcorp"),
];

pub fn alias_for(symbol: &str) -> Option<&'static str> {
    ALIASES.iter().find(|(s, _)| *s == symbol).map(|(_, n)| *n)
}

/// Trailing LST boilerplate, longest-first so "Staked SOL" wins over "SOL".
const BOILERPLATE: &[&str] = &[
    "liquid staked solana", "liquid staked sol",
    "staked solana", "wrapped solana", "restaked solana",
    "staked sol", "wrapped sol", "restaked sol",
    "solana", "sol",
];

pub fn slugify(sanctum_name: &str) -> Slug {
    let lower = sanctum_name.trim().to_lowercase();
    let mut core = lower.as_str();
    let mut stripped = false;
    let mut risky = false;
    for suffix in BOILERPLATE {
        if let Some(rest) = core.strip_suffix(suffix) {
            let rest = rest.trim_end();
            // The whole name was boilerplate ("Wrapped SOL", "SOL"): nothing
            // legitimate survives, so refuse rather than emitting the unstripped
            // name as if it were a brand.
            if rest.is_empty() {
                return Slug::Unusable;
            }
            core = rest;
            stripped = true;
            // "Staked SOL" and its siblings are the canonical, unambiguous LST
            // suffix. "Wrapped"/"Restaked"/bare "SOL" double as real brand
            // words, so a short result from those needs a human to confirm.
            risky = !matches!(
                *suffix,
                "liquid staked solana" | "liquid staked sol" | "staked solana" | "staked sol"
            );
            break;
        }
    }

    let slug: String = {
        let mut out = String::new();
        let mut pending_sep = false;
        for ch in core.chars() {
            if ch.is_ascii_alphanumeric() {
                if pending_sep && !out.is_empty() {
                    out.push('_');
                }
                pending_sep = false;
                out.push(ch);
            } else {
                pending_sep = true;
            }
        }
        out
    };

    if slug.len() < 3 {
        return Slug::Unusable;
    }
    // A risky strip that left a single short token deserves a second look.
    if stripped && risky && !slug.contains('_') && slug.len() <= 5 {
        return Slug::Suspicious { slug, original: sanctum_name.to_string() };
    }
    Slug::Clean(slug)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --features discover --lib`
Expected: PASS, all slug tests green

- [ ] **Step 5: Commit**

```bash
git add src/discovery.rs
git commit -m "feat(discovery): entity-name slug rule with brand guards and aliases"
```

---

### Task 4: Registry parse and first-run bootstrap

**Files:**
- Modify: `src/discovery.rs`

**Interfaces:**
- Consumes: `StakePool` (Task 2)
- Produces:
  - `pub struct Entry { pub name: String, pub authority: String, pub note: Option<String> }`
  - `pub struct Registry { pub manual: Vec<Entry>, pub generated: Vec<Entry>, pub retired: Vec<Entry> }`
  - `pub fn parse_registry(src: &str) -> Result<Registry, String>`
  - `pub fn promote_derivable(reg: &mut Registry, derived: &HashSet<String>)`

Task 6 creates the section markers by hand, so the generator never has to invent them. `promote_derivable` moves any MANUAL entry whose authority turns out to be derivable into GENERATED — that is the first-run 33/28 split, and it stays correct on every later run. `derived` is the **pre-threshold** set; using the post-threshold set would leave a known-but-small SPL pool stranded in MANUAL forever.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn parses_a_markered_file() {
    let src = r#"
        // ---- MANUAL ----
        PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"),
        // ---- END MANUAL ----
        // ---- GENERATED ----
        PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        // ---- END GENERATED ----
        // ---- RETIRED ----
        PoolInfo::new("socean", "AzZRvyyMHBm8EHEksWxq4ozFL7JxLMydCDMGhqM6BVck"),
        // ---- END RETIRED ----
    "#;
    let r = parse_registry(src).unwrap();
    assert_eq!(r.manual.len(), 1);
    assert_eq!(r.generated[0].name, "jito");
    assert_eq!(r.retired[0].name, "socean");
}

#[test]
fn promotes_derivable_manual_entries() {
    // First run: Task 6 parked all 61 existing entries in MANUAL. The 28 whose
    // authorities are derivable belong in GENERATED.
    let src = r#"
        // ---- MANUAL ----
        PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"),
        // ---- END MANUAL ----
        // ---- GENERATED ----
        // ---- END GENERATED ----
        // ---- RETIRED ----
        // ---- END RETIRED ----
    "#;
    let mut derived = HashSet::new();
    derived.insert("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS".to_string());

    let mut r = parse_registry(src).unwrap();
    promote_derivable(&mut r, &derived);
    assert_eq!(r.generated.iter().map(|e| &e.name).collect::<Vec<_>>(), vec!["jito"]);
    assert_eq!(r.manual.iter().map(|e| &e.name).collect::<Vec<_>>(), vec!["kraken"]);
}

#[test]
fn rejects_duplicate_authority_across_sections() {
    // POOLS_BY_AUTHORITY is built with collect(), so a duplicate would silently
    // overwrite and misattribute every stake account fetched for it.
    let src = r#"
        // ---- MANUAL ----
        PoolInfo::new("a", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        // ---- END MANUAL ----
        // ---- GENERATED ----
        PoolInfo::new("b", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        // ---- END GENERATED ----
    "#;
    let err = parse_registry(src).unwrap_err();
    assert!(err.contains("duplicate authority"), "got: {err}");
}
```

Add `use std::collections::HashSet;` to the test module imports.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --features discover registry`
Expected: FAIL — `cannot find function parse_registry`

- [ ] **Step 3: Write the implementation**

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub authority: String,
    pub note: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Registry {
    pub manual: Vec<Entry>,
    pub generated: Vec<Entry>,
    pub retired: Vec<Entry>,
}

impl Registry {
    pub fn all(&self) -> impl Iterator<Item = &Entry> {
        self.manual.iter().chain(&self.generated).chain(&self.retired)
    }
    /// Authority -> name, the binding the freeze rule protects.
    pub fn frozen_names(&self) -> HashMap<String, String> {
        self.all().map(|e| (e.authority.clone(), e.name.clone())).collect()
    }
}

fn parse_entry(line: &str) -> Option<Entry> {
    let rest = line.trim().strip_prefix("PoolInfo::new(")?;
    let mut parts = rest.split('"').skip(1).step_by(2);
    let name = parts.next()?.to_string();
    let authority = parts.next()?.to_string();
    Some(Entry { name, authority, note: None })
}

pub fn parse_registry(src: &str) -> Result<Registry, String> {
    if !src.contains("---- GENERATED") {
        return Err("no section markers in src/pools.rs; run Task 6 first".into());
    }
    let mut reg = Registry::default();
    {
        let mut section = None;
        for line in src.lines() {
            let t = line.trim();
            if t.contains("---- MANUAL") && !t.contains("END") { section = Some(0); continue; }
            if t.contains("---- GENERATED") && !t.contains("END") { section = Some(1); continue; }
            if t.contains("---- RETIRED") && !t.contains("END") { section = Some(2); continue; }
            if t.contains("END MANUAL") || t.contains("END GENERATED") || t.contains("END RETIRED") {
                section = None;
                continue;
            }
            if let (Some(s), Some(e)) = (section, parse_entry(t)) {
                match s {
                    0 => reg.manual.push(e),
                    1 => reg.generated.push(e),
                    _ => reg.retired.push(e),
                }
            }
        }
    }

    let mut seen = HashSet::new();
    for e in reg.all() {
        if !seen.insert(e.authority.clone()) {
            return Err(format!("duplicate authority {} (name {})", e.authority, e.name));
        }
    }
    Ok(reg)
}

/// Move MANUAL entries whose authority is derivable into GENERATED. On the
/// first run this performs the 33/28 split; afterwards it promotes any
/// hand-added pool that turns out to be an SPL-family pool.
pub fn promote_derivable(reg: &mut Registry, derived: &HashSet<String>) {
    let (promote, keep): (Vec<_>, Vec<_>) =
        reg.manual.drain(..).partition(|e| derived.contains(&e.authority));
    if !promote.is_empty() {
        eprintln!("promoting {} manual entries to generated", promote.len());
    }
    reg.manual = keep;
    reg.generated.extend(promote);
}
```

Add `use std::collections::HashMap;` at the top of `src/discovery.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --features discover --lib`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/discovery.rs
git commit -m "feat(discovery): registry parsing with first-run bootstrap"
```

---

### Task 5: Name assignment and deterministic rendering

**Files:**
- Modify: `src/discovery.rs`

**Interfaces:**
- Consumes: `Registry`, `Entry`, `Slug`, `slugify`, `alias_for`, `StakePool`
- Produces:
  - `pub struct Candidate { pub authority: String, pub pool: String, pub mint: String, pub sanctum_name: Option<String>, pub sanctum_symbol: Option<String>, pub total_sol: f64, pub stale: bool }`
  - `pub fn assign_names(reg: &Registry, candidates: &[Candidate]) -> Registry` — also demotes previously-generated pools that are no longer candidates into `retired`
  - `pub fn splice(existing: &str, reg: &Registry, provenance: &str) -> String`

- [ ] **Step 1: Write the failing tests**

```rust
fn cand(auth: &str, pool: &str, name: Option<&str>) -> Candidate {
    Candidate {
        authority: auth.into(),
        pool: pool.into(),
        mint: "MintMintMintMintMintMintMintMintMintMintMin".into(),
        sanctum_name: name.map(String::from),
        sanctum_symbol: None,
        total_sol: 100.0,
        stale: false,
    }
}

#[test]
fn existing_names_are_frozen_even_when_onchain_disagrees() {
    // forward_industries' pool is "Dum Staked SOL" on-chain. Renaming it would
    // break every consumer keyed on the old name.
    let mut reg = Registry::default();
    reg.generated.push(Entry {
        name: "forward_industries".into(),
        authority: "3ndMuPC9Cz5VC4RJkpoPaZz6Px6eVXtRenw9Yi1o2xnA".into(),
        note: None,
    });
    let out = assign_names(
        &reg,
        &[cand("3ndMuPC9Cz5VC4RJkpoPaZz6Px6eVXtRenw9Yi1o2xnA", "PoolA", Some("Dum Staked SOL"))],
    );
    assert_eq!(out.generated[0].name, "forward_industries");
}

#[test]
fn new_pools_get_slugged_names() {
    let out = assign_names(
        &Registry::default(),
        &[cand("AuthA", "PoolA", Some("Phantom Staked SOL"))],
    );
    assert_eq!(out.generated[0].name, "phantom");
}

#[test]
fn unnamed_pools_fall_back_to_pool_prefix() {
    let out = assign_names(&Registry::default(), &[cand("AuthA", "PoolAddr12345", None)]);
    assert_eq!(out.generated[0].name, "unnamed_PoolAddr");
}

#[test]
fn pools_that_drop_out_are_retired_not_deleted() {
    // Global constraint: a name, once emitted, is never deleted. A pool that
    // falls below --min-sol or leaves the chain moves to RETIRED, where
    // get_pool_by_name() still resolves it.
    let mut reg = Registry::default();
    reg.generated.push(Entry { name: "gone".into(), authority: "GoneAuth".into(), note: None });
    reg.generated.push(Entry { name: "stays".into(), authority: "StaysAuth".into(), note: None });

    let out = assign_names(&reg, &[cand("StaysAuth", "PoolS", Some("Jito Staked SOL"))]);

    assert_eq!(out.generated.len(), 1);
    assert_eq!(out.generated[0].name, "stays");
    assert_eq!(out.retired.len(), 1);
    assert_eq!(out.retired[0].name, "gone", "dropped pool must be retired, never deleted");
    assert!(out.retired[0].note.as_deref().unwrap().contains("below threshold"));
}

#[test]
fn retired_pools_keep_their_names_reserved() {
    // A retired name must stay taken, or a new pool could claim it and two
    // different authorities would answer to the same API key over time.
    let mut reg = Registry::default();
    reg.retired.push(Entry { name: "phantom".into(), authority: "OldAuth".into(), note: None });

    let out = assign_names(&reg, &[cand("NewAuth", "PoolN", Some("Phantom Staked SOL"))]);
    assert_eq!(out.generated[0].name, "phantom_2");
}

#[test]
fn collisions_suffix_deterministically_by_authority() {
    // Suffix order must not depend on input order, or _2/_3 could swap between
    // runs and break the freeze invariant for the pools it exists to protect.
    let a = cand("Bbb", "Pool1", Some("Sanctum Staked SOL"));
    let b = cand("Aaa", "Pool2", Some("Sanctum Staked SOL"));

    let fwd = assign_names(&Registry::default(), &[a.clone(), b.clone()]);
    let rev = assign_names(&Registry::default(), &[b, a]);

    let names = |r: &Registry| {
        let mut v: Vec<_> = r.generated.iter().map(|e| (e.authority.clone(), e.name.clone())).collect();
        v.sort();
        v
    };
    assert_eq!(names(&fwd), names(&rev));
    assert_eq!(names(&fwd), vec![
        ("Aaa".to_string(), "sanctum".to_string()),
        ("Bbb".to_string(), "sanctum_2".to_string()),
    ]);
}

#[test]
fn splice_is_deterministic_and_sorted() {
    let mut reg = Registry::default();
    reg.generated.push(Entry { name: "b".into(), authority: "Zzz".into(), note: None });
    reg.generated.push(Entry { name: "a".into(), authority: "Aaa".into(), note: None });
    let existing = "// ---- GENERATED ----\n// ---- END GENERATED ----\n";
    let out = splice(existing, &reg, "epoch 1028");
    assert_eq!(out, splice(existing, &reg, "epoch 1028"));
    assert!(out.find("Aaa").unwrap() < out.find("Zzz").unwrap(), "sorted by authority");
}

#[test]
fn splice_preserves_everything_outside_the_markers() {
    // The accessors, the HashMap indexes and the existing test module all live
    // outside the markers. Rewriting the whole file would delete them.
    let existing = concat!(
        "pub fn get_pool_by_name(n: &str) -> Option<&PoolInfo> { todo!() }\n",
        "// ---- GENERATED ----\n",
        "        PoolInfo::new(\"old\", \"OldAuth\"),\n",
        "// ---- END GENERATED ----\n",
        "#[cfg(test)] mod tests { }\n",
    );
    let mut reg = Registry::default();
    reg.generated.push(Entry { name: "new".into(), authority: "NewAuth".into(), note: None });

    let out = splice(existing, &reg, "epoch 1028");
    assert!(out.contains("get_pool_by_name"), "accessors must survive");
    assert!(out.contains("#[cfg(test)] mod tests"), "tests must survive");
    assert!(out.contains("NewAuth") && !out.contains("OldAuth"), "body replaced");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --features discover assign`
Expected: FAIL — `cannot find function assign_names`

- [ ] **Step 3: Write the implementation**

```rust
#[derive(Debug, Clone)]
pub struct Candidate {
    pub authority: String,
    pub pool: String,
    pub mint: String,
    pub sanctum_name: Option<String>,
    pub sanctum_symbol: Option<String>,
    pub total_sol: f64,
    pub stale: bool,
}

pub fn assign_names(reg: &Registry, candidates: &[Candidate]) -> Registry {
    let frozen = reg.frozen_names();
    let mut taken: HashSet<String> = reg.all().map(|e| e.name.clone()).collect();

    // Deterministic: sort by authority before handing out any suffix.
    let mut sorted = candidates.to_vec();
    sorted.sort_by(|a, b| a.authority.cmp(&b.authority));

    let mut out = reg.clone();
    out.generated.clear();

    // A previously-generated pool that is no longer a candidate has dropped
    // below the threshold or left the chain. It is retired, never deleted:
    // deleting it would remove a live API key.
    let present: HashSet<&str> = sorted.iter().map(|c| c.authority.as_str()).collect();
    for e in reg.generated.iter().filter(|e| !present.contains(e.authority.as_str())) {
        out.retired.push(Entry {
            name: e.name.clone(),
            authority: e.authority.clone(),
            note: Some("below threshold or no longer on-chain".into()),
        });
    }

    for c in sorted {
        if let Some(existing) = frozen.get(&c.authority) {
            let mut e = Entry { name: existing.clone(), authority: c.authority.clone(), note: None };
            if c.stale {
                e.note = Some("stale: cached balance".into());
            }
            out.generated.push(e);
            continue;
        }

        let mut note = None;
        let base = c
            .sanctum_symbol
            .as_deref()
            .and_then(alias_for)
            .map(String::from)
            .or_else(|| match c.sanctum_name.as_deref().map(slugify) {
                Some(Slug::Clean(s)) => Some(s),
                Some(Slug::Suspicious { slug, original }) => {
                    note = Some(format!("verify: from {original:?}"));
                    Some(slug)
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                note = Some("TODO: name".into());
                format!("unnamed_{}", &c.pool[..c.pool.len().min(8)])
            });

        let mut name = base.clone();
        let mut n = 2;
        while taken.contains(&name) {
            name = format!("{base}_{n}");
            n += 1;
        }
        taken.insert(name.clone());
        out.generated.push(Entry { name, authority: c.authority.clone(), note });
    }
    out
}

fn body(entries: &[Entry]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.authority.cmp(&b.authority));
    sorted
        .iter()
        .map(|e| {
            format!(
                "        PoolInfo::new(\"{}\", \"{}\"),{}\n",
                e.name,
                e.authority,
                e.note.as_ref().map(|n| format!(" // {n}")).unwrap_or_default()
            )
        })
        .collect()
}

/// Replace the text between each `---- X ----` / `---- END X ----` pair,
/// leaving every byte outside the markers untouched. The accessors, the
/// HashMap indexes and the test module all live outside them.
fn replace_region(src: &str, tag: &str, new_body: &str) -> String {
    let open = format!("---- {tag}");
    let close = format!("---- END {tag}");
    let (Some(o), Some(c)) = (src.find(&open), src.find(&close)) else { return src.to_string() };
    let body_start = src[o..].find('\n').map(|i| o + i + 1).unwrap_or(o);
    let line_start = src[..c].rfind('\n').map(|i| i + 1).unwrap_or(c);
    format!("{}{}{}", &src[..body_start], new_body, &src[line_start..])
}

pub fn splice(existing: &str, reg: &Registry, provenance: &str) -> String {
    let mut out = replace_region(existing, "MANUAL", &body(&reg.manual));
    out = replace_region(&out, "GENERATED", &body(&reg.generated));
    out = replace_region(&out, "RETIRED", &body(&reg.retired));

    // Provenance lives on one comment line that is rewritten in place.
    let stamp = format!("//! Provenance: {provenance}");
    match out.lines().position(|l| l.starts_with("//! Provenance:")) {
        Some(i) => {
            let mut lines: Vec<&str> = out.lines().collect();
            lines[i] = &stamp;
            lines.join("\n") + "\n"
        }
        None => format!("{stamp}\n{out}"),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --features discover --lib`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/discovery.rs
git commit -m "feat(discovery): name assignment with freeze rule and marker splicing"
```

---

### Task 6: Runtime split — active vs retired pools

**Files:**
- Modify: `src/pools.rs:30-157`

**Interfaces:**
- Consumes: nothing from the generator (this is runtime code)
- Produces: `pub fn get_active_pools() -> &'static [PoolInfo]`

`get_all_pools()`, `get_pool_by_name()`, `get_pool_by_authority()` keep their exact current signatures and behavior.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/pools.rs`:

```rust
#[test]
fn retired_pools_still_resolve_by_name() {
    // Retiring a pool must never break an API key. It leaves fetch_all_pools,
    // but name lookup keeps working.
    for p in POOLS_RETIRED.iter() {
        assert!(get_pool_by_name(&p.name).is_some(), "{} stopped resolving", p.name);
    }
}

#[test]
fn active_pools_exclude_retired() {
    let active: std::collections::HashSet<_> = get_active_pools().iter().map(|p| &p.name).collect();
    for p in POOLS_RETIRED.iter() {
        assert!(!active.contains(&p.name), "{} should not be active", p.name);
    }
    assert_eq!(get_all_pools().len(), get_active_pools().len() + POOLS_RETIRED.len());
}

#[test]
fn all_pools_have_unique_authorities() {
    // POOLS_BY_AUTHORITY is built with collect(), so duplicates silently
    // overwrite and misattribute fetched stake accounts.
    let mut seen = std::collections::HashSet::new();
    for p in get_all_pools() {
        assert!(seen.insert(&p.authority), "duplicate authority: {} ({})", p.authority, p.name);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test pools::tests`
Expected: FAIL — `cannot find value POOLS_RETIRED`, `cannot find function get_active_pools`

- [ ] **Step 3: Implement the split and create the section markers**

In `src/pools.rs`, rename the existing `POOLS_REGISTRY` literal to `POOLS_ACTIVE`. Wrap all 61 existing entries in MANUAL markers and add empty GENERATED and RETIRED regions — the generator splices into these and never has to invent them. It will promote the 28 derivable entries out of MANUAL on its first run.

```rust
static POOLS_ACTIVE: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    vec![
        // ---- MANUAL: custodial and non-SPL pools. Edit freely. ----
        PoolInfo::new("foundation", "mpa4abUkjQoAvPzREkh5Mo75hZhPFQ2FSH6w7dWKuQ5"),
        // ... all 61 existing entries, unchanged ...
        // ---- END MANUAL ----

        // ---- GENERATED: written by discover_pools. Do not edit. ----
        // ---- END GENERATED ----
    ]
});

static POOLS_RETIRED: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    vec![
        // ---- RETIRED: kept so existing names resolve. Excluded from fetch_all_pools. ----
        // ---- END RETIRED ----
    ]
});
```

Also add a provenance line near the top of the file so the generator has one to rewrite:

```rust
//! Provenance: hand-maintained; not yet generated
```

Then compose the two statics:

```rust
/// Every pool, active and retired. Retired entries stay here so existing
/// pool names keep resolving.
static POOLS_REGISTRY: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    POOLS_ACTIVE.iter().chain(POOLS_RETIRED.iter()).cloned().collect()
});

/// Pools that should actually be fetched. Excludes retired entries.
#[must_use]
pub fn get_active_pools() -> &'static [PoolInfo] {
    &POOLS_ACTIVE
}
```

`get_all_pools()` continues to return `&POOLS_REGISTRY` — unchanged.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test pools::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/pools.rs
git commit -m "feat(pools): split active from retired; add get_active_pools"
```

---

### Task 7: Tolerate pools with no stake accounts

**Files:**
- Modify: `src/client.rs:176-180` (`fetch_all_pools`), `src/client.rs:315-330` (`fetch_single_pool_impl`)

**Interfaces:**
- Consumes: `get_active_pools()` (Task 6)
- Produces: no signature changes

- [ ] **Step 1: Write the failing test**

Add to the `tests` module at the top of `src/client.rs`:

```rust
#[test]
fn empty_stake_accounts_produce_empty_statistics_not_an_error() {
    // 81 of the 261 discovered pools hold under 10 SOL and routinely have zero
    // stake accounts (everything sits in reserve). That is a healthy pool, not
    // a failure — but it must stay visible, because an empty result is also
    // what a mis-derived authority looks like.
    let stats = PoolsDataClient::calculate_pool_statistics(&[]);
    assert_eq!(stats.total_accounts, 0);
    assert_eq!(stats.total_lamports, 0);
    assert_eq!(stats.validator_count, 0);
}
```

- [ ] **Step 2: Run test to verify it passes or fails**

Run: `cargo test client::tests::empty_stake_accounts`
Expected: PASS (this pins existing behavior before the change below)

- [ ] **Step 3: Remove the error path and log instead**

In `src/client.rs`, replace the empty-list guard in `fetch_single_pool_impl`:

```rust
            Ok(stake_accounts) => {
                if stake_accounts.is_empty() {
                    // Legitimate for reserve-only pools, but also what a bad
                    // authority looks like. Never silent.
                    log::warn!(
                        "pool {} ({}) returned no stake accounts",
                        pool_info.name,
                        pool_info.authority
                    );
                }
```

Delete the `return Err(PoolError::new(... NoStakeAccounts ...))` block that followed. The rest of the arm (validator distribution, statistics, `PoolData`) is unchanged and now runs for empty input too.

- [ ] **Step 4: Point fetch_all_pools at active pools only**

In `src/client.rs:176-180`:

```rust
    pub async fn fetch_all_pools(&self) -> Result<HashMap<String, ProductionPoolData>> {
        let all_pools = get_active_pools();
        let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
        self.fetch_pools(&pool_names).await
    }
```

Update the import on `src/client.rs:62` to bring in `get_active_pools`. Apply the same change in `fetch_all_pools_with_stats` (`src/client.rs:91`).

- [ ] **Step 5: Verify the whole suite**

Run: `cargo test`
Expected: PASS. `NoStakeAccounts` may now be unused in `src/error.rs:91` — leave the variant in place (removing it is a breaking API change) but confirm no warnings block the build.

- [ ] **Step 6: Commit**

```bash
git add src/client.rs
git commit -m "fix(client): treat reserve-only pools as success, not NoStakeAccounts"
```

---

### Task 8: The generator binary

**Files:**
- Create: `examples/discover_pools.rs`

**Interfaces:**
- Consumes: everything from `crate::discovery`
- Produces: the executable

- [ ] **Step 1: Write the generator**

```rust
//! Regenerates src/pools.rs from on-chain stake pool programs.
//!
//! Run: cargo run --features discover --example discover_pools -- --min-sol 1

use solana_pools_data_lib::discovery::*;
use solana_pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

const SANCTUM_LIST: &str =
    "https://raw.githubusercontent.com/igneous-labs/sanctum-lst-list/master/sanctum-lst-list.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let min_sol: f64 = args.iter().position(|a| a == "--min-sol")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(1.0);
    let verify = args.iter().any(|a| a == "--verify");
    let rpc = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let out_path = "src/pools.rs";

    let http = reqwest::Client::new();
    let epoch = get_epoch(&http, &rpc).await?;

    // 1. Enumerate all three programs. A program returning zero accounts means
    //    the layout or program ID assumption broke, not that pools vanished.
    let mut by_program: HashMap<&str, Vec<StakePool>> = HashMap::new();
    for program in [SPL_STAKE_POOL, SANCTUM_SPL, SANCTUM_SPL_MULTI] {
        let pools = fetch_stake_pools(&http, &rpc, program).await?;
        if pools.is_empty() {
            return Err(format!("program {program} returned no StakePool accounts; \
                                layout or program ID assumption has broken").into());
        }
        eprintln!("{program}: {} pools", pools.len());
        by_program.insert(program, pools);
    }

    // 2. Derive authorities. Pre-threshold set is what bootstrap classifies against.
    let mut cohorts: HashMap<&str, Vec<Candidate>> = HashMap::new();
    let mut all_derived: HashSet<String> = HashSet::new();
    let sanctum = fetch_sanctum(&http).await?;

    for (program, pools) in &by_program {
        let prog = Pubkey::from_str(program)?;
        let mut cohort = Vec::new();
        for sp in pools {
            let authority = match derive_authority(&sp.pool, sp.bump, &prog) {
                Ok(a) => a.to_string(),
                Err(e) => { eprintln!("skip {}: {e}", sp.pool); continue; }
            };
            all_derived.insert(authority.clone());

            let stale = epoch.saturating_sub(sp.last_update_epoch) > 10;
            let mut total_sol = sp.total_lamports as f64 / 1e9;

            // A stale cached balance may not admit a NEW entry on its own.
            if stale && total_sol >= min_sol {
                total_sol = live_stake_sol(&http, &rpc, &authority).await?;
            }
            if total_sol < min_sol { continue; }

            let (name, symbol) = sanctum.get(&sp.mint.to_string())
                .cloned().map(|(n, s)| (Some(n), Some(s))).unwrap_or((None, None));
            cohort.push(Candidate {
                authority, pool: sp.pool.to_string(), mint: sp.mint.to_string(),
                sanctum_name: name, sanctum_symbol: symbol, total_sol, stale,
            });
        }
        cohorts.insert(program, cohort);
    }

    let existing = std::fs::read_to_string(out_path)?;
    let mut reg = parse_registry(&existing)?;
    // First run: moves the 28 derivable entries out of MANUAL into GENERATED.
    promote_derivable(&mut reg, &all_derived);
    let known: HashSet<String> = reg.all().map(|e| e.authority.clone()).collect();

    // 3. Verification is per program cohort. A global "all empty" check would
    //    never fire when only one program diverges, and that program's wrong
    //    authorities would be emitted anyway.
    let mut candidates = Vec::new();
    for (program, cohort) in cohorts {
        let fresh: Vec<_> = cohort.iter().filter(|c| !known.contains(&c.authority)).collect();
        if verify && !fresh.is_empty() {
            let mut nonempty = 0;
            for c in &fresh {
                if stake_account_count(&http, &rpc, &c.authority).await? > 0 { nonempty += 1; }
            }
            if nonempty == 0 {
                eprintln!("ABORT: all {} new authorities under {program} returned no stake \
                           accounts; derivation for this program looks broken", fresh.len());
                std::process::exit(1);
            }
        }
        candidates.extend(cohort);
    }

    // 4. Assign names, render, write atomically.
    let provenance = format!(
        "{rpc} epoch {epoch}, --min-sol {min_sol}, {} pools", candidates.len()
    );
    let updated = assign_names(&reg, &candidates);
    let tmp = format!("{out_path}.tmp");
    std::fs::write(&tmp, splice(&existing, &updated, &provenance))?;
    std::fs::rename(&tmp, out_path)?;
    eprintln!("wrote {out_path}: {} manual, {} generated, {} retired",
        updated.manual.len(), updated.generated.len(), updated.retired.len());
    Ok(())
}
```

Implement the four RPC helpers (`get_epoch`, `fetch_stake_pools`, `live_stake_sol`, `stake_account_count`) and `fetch_sanctum` in the same file. `fetch_stake_pools` posts `getProgramAccounts` with `filters: [{"dataSize":611},{"memcmp":{"offset":0,"bytes":"2","encoding":"base58"}}]` and base64 encoding, then calls `decode` per account. `stake_account_count` posts `getProgramAccounts` against `Stake11111111111111111111111111111111111111` with `dataSlice {offset:0,length:0}` and `memcmp {offset:12, bytes:<authority>}`, returning the array length. `live_stake_sol` uses the same filter with `dataSlice {offset:156,length:8}` and sums the little-endian `u64` stake values.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --features discover --example discover_pools`
Expected: compiles clean

- [ ] **Step 3: Dry-run against mainnet**

```bash
cp src/pools.rs /tmp/pools.rs.bak
cargo run --features discover --example discover_pools -- --min-sol 1
```

Expected on stderr: three per-program counts (~232 / ~1401 / ~56), a line reading `promoting 28 manual entries to generated`, and a final write line.

- [ ] **Step 4: Verify the output compiles and the registry is sane**

```bash
cargo test
git diff --stat src/pools.rs
```

Expected: tests pass, including `all_pools_have_unique_authorities`. Registry grows from 61 to roughly 261 active entries.

- [ ] **Step 5: Verify idempotency**

```bash
cargo run --features discover --example discover_pools -- --min-sol 1
git diff src/pools.rs | wc -l
```

Expected: `0` — a second run against unchanged chain state produces a byte-identical file. If not, the sort or the provenance line is non-deterministic; fix before committing.

- [ ] **Step 6: Commit the generator only**

```bash
git checkout src/pools.rs   # registry lands in Task 9, after human review
git add examples/discover_pools.rs
git commit -m "feat(discovery): add offline pool discovery generator"
```

---

### Task 9: Generate and review the real registry

**Files:**
- Modify: `src/pools.rs` (generated content)
- Modify: `README.md:88-89`

- [ ] **Step 1: Generate with verification on**

```bash
cargo run --features discover --example discover_pools -- --min-sol 1 --verify
```

- [ ] **Step 2: Review the diff by hand**

```bash
git diff src/pools.rs | grep '^+' | grep -c PoolInfo   # ~261
git diff src/pools.rs | grep 'TODO: name' | wc -l      # ~88 placeholders
git diff src/pools.rs | grep 'verify: from'            # short strips to eyeball
git diff src/pools.rs | grep -E '^-.*PoolInfo'         # MUST be empty
```

The last check is the important one: **no existing entry may disappear or change**. Any `-` line for a `PoolInfo` that is not matched by an identical `+` line is a freeze-rule violation. Stop and fix the generator if one appears.

- [ ] **Step 3: Name what you can**

Edit the `unnamed_*` entries you recognize directly in the GENERATED block. The generator will freeze whatever you write on the next run. Known from earlier investigation: `sctmSOL` is Sanctum, `PSOL` is Phantom, `GTSOL` is gate.io, `dfdvSOL` is DeFi Development Corp.

- [ ] **Step 4: Verify the library still works end to end**

```bash
cargo test
cargo run --example quick_test
```

Expected: tests pass; `quick_test` fetches successfully. Reserve-only pools log a `warn` rather than failing.

- [ ] **Step 5: Update the README pool count**

`README.md:4` and `README.md:88-89` both claim "31 pools". Replace with the real count and note that the list is generated:

```markdown
## Supported Pools
Auto-discovered SPL-family stake pools plus a hand-maintained list of custodial and
non-SPL stakers. Regenerate with:
`cargo run --features discover --example discover_pools -- --min-sol 1`
List: `PoolsDataClient::list_available_pools()`
```

- [ ] **Step 6: Commit**

```bash
git add src/pools.rs README.md
git commit -m "feat(pools): regenerate registry from on-chain discovery

61 -> ~261 pools. Existing names frozen by authority pubkey; no entry
renamed or removed."
```

---

## Self-Review

**Spec coverage:**

| Spec section | Task |
|---|---|
| Discovery: 3 programs, offsets, PDA with off-curve check | 2, 8 |
| Naming: `name` not `symbol`, guards, aliases | 3 |
| Name precedence and freeze | 5 |
| Deterministic collision suffixing | 5 |
| Retention / RETIRED section / `get_active_pools` | 5, 6 |
| Threshold and `total_lamports` staleness | 8 |
| MSRV-gated dependencies | 1 |
| First-run bootstrap (pre-threshold set) | 4 |
| `--verify` per-program cohort | 8 |
| Duplicate authority detection | 4, 6 |
| Determinism and provenance | 5, 8 |
| `client.rs` empty-pool tolerance | 7 |

**Not carried into code:** the deferred 256-bucket census (spec Appendix) is explicitly out of scope.

**Known follow-ups, deliberately deferred:**
- `src/client.rs` has a duplicate epoch-unaware `calculate_pool_statistics` shadowing the epoch-aware one in `src/types.rs:8`. Untouched here; unrelated to discovery.
- `Migration` (same mint, new authority) is only flagged to stderr. Merging is a human edit by design.
