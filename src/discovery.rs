//! Offline pool discovery. Compiled only with the `discover` feature.

use std::collections::HashMap;
use std::collections::HashSet;

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
            // The whole name was boilerplate: stripping consumed it, nothing
            // legitimate is left to slug. Doc comment on Unusable says exactly
            // this case.
            if rest.is_empty() {
                return Slug::Unusable;
            }
            core = rest;
            stripped = true;
            // "Staked SOL" (and its "liquid staked"/"staked solana" siblings) is
            // the canonical, unambiguous LST suffix. "Wrapped"/"Restaked"/bare
            // "SOL"/"Solana" are looser matches that can double as real brand
            // words, so a short result from those needs a second look.
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
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("PoolInfo::new(")?;
    let mut parts = rest.split('"').skip(1).step_by(2);
    let name = parts.next()?.to_string();
    let authority = parts.next()?.to_string();
    // A trailing `// comment` is hand-maintained context (e.g. "exchange,
    // verified 2026-01"). It must survive a parse-then-splice round trip or
    // the generator silently deletes it on the next run.
    let note = trimmed.find("//").map(|i| trimmed[i + 2..].trim().to_string());
    Some(Entry { name, authority, note })
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

/// Trailing-comment components the generator owns and re-derives on every run.
/// Everything else in a note — `TODO: name`, `verify: from ...`, a reviewer's
/// hand-written annotation — belongs to whoever wrote it and is carried across
/// untouched.
const STALE_NOTE: &str = "stale: cached balance";
const RETIRED_NOTE: &str = "below threshold or no longer on-chain";
const MANAGED_NOTES: &[&str] = &[STALE_NOTE, RETIRED_NOTE];

/// Rebuild a note as `; `-separated components: everything the previous run
/// left behind, minus the generator-owned markers, plus the ones that apply now.
///
/// Replacing the note outright instead would make the generator non-idempotent:
/// a pool that gets `// TODO: name` on run N is *frozen* on run N+1, and the
/// frozen path would emit it with no note at all — silently deleting exactly
/// the annotations a reviewer works from. Re-deriving the managed markers is
/// what lets `stale:` disappear again once a pool's balance is fresh.
fn merge_note(previous: Option<&str>, add: &[&str]) -> Option<String> {
    let mut parts: Vec<&str> = previous
        .map(|n| {
            n.split(';')
                .map(str::trim)
                .filter(|p| !p.is_empty() && !MANAGED_NOTES.contains(p))
                .collect()
        })
        .unwrap_or_default();
    parts.extend(add);
    (!parts.is_empty()).then(|| parts.join("; "))
}

pub fn assign_names(reg: &Registry, candidates: &[Candidate]) -> Registry {
    let frozen = reg.frozen_names();
    let prev_notes: HashMap<&str, &str> = reg
        .all()
        .filter_map(|e| e.note.as_deref().map(|n| (e.authority.as_str(), n)))
        .collect();
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
            note: merge_note(e.note.as_deref(), &[RETIRED_NOTE]),
        });
    }

    // A pool that dips below the threshold for one epoch and comes back is a
    // candidate again. It must be promoted back out of retired (and out of
    // manual, if that's where it was hand-added) before being pushed into
    // generated below — otherwise it lives in two sections at once, which
    // parse_registry rejects as a duplicate authority on the very next run
    // and which double-counts it in get_all_pools() today.
    out.retired.retain(|e| !present.contains(e.authority.as_str()));
    out.manual.retain(|e| !present.contains(e.authority.as_str()));

    for c in sorted {
        let now: &[&str] = if c.stale { &[STALE_NOTE] } else { &[] };
        if let Some(existing) = frozen.get(&c.authority) {
            out.generated.push(Entry {
                name: existing.clone(),
                authority: c.authority.clone(),
                note: merge_note(prev_notes.get(c.authority.as_str()).copied(), now),
            });
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
        out.generated.push(Entry {
            name,
            authority: c.authority.clone(),
            note: merge_note(note.as_deref(), now),
        });
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

/// True if `line`, once trimmed and stripped of a leading `//`, `///`, or
/// `//!` comment prefix, IS exactly `marker` (e.g. `---- GENERATED ----`). A
/// line that merely *mentions* the marker text inside a longer sentence — a
/// header, a doc comment explaining the format — is not equal to it, so a
/// decoy mention can never be mistaken for the real marker.
fn is_marker_line(line: &str, marker: &str) -> bool {
    let t = line.trim();
    let rest = t.strip_prefix("//!").or_else(|| t.strip_prefix("///")).or_else(|| t.strip_prefix("//"));
    rest.is_some_and(|r| r.trim() == marker)
}

/// Find the marker line and return the byte offsets bounding it (through
/// its own trailing newline, if it has one).
fn find_marker_line(src: &str, marker: &str) -> Option<(usize, usize)> {
    let mut offset = 0;
    for piece in src.split_inclusive('\n') {
        if is_marker_line(piece, marker) {
            return Some((offset, offset + piece.len()));
        }
        offset += piece.len();
    }
    None
}

/// Replace the text between a `---- X ----` marker line and its matching
/// `---- END X ----` marker line, leaving every byte outside that span
/// untouched. The accessors, the HashMap indexes and the test module all
/// live outside them.
///
/// The end marker is searched for only in the text *after* the start
/// marker, so a stray or out-of-order end marker can never be paired with
/// it. Either a missing marker or an out-of-order pair fails safe: `src` is
/// returned unchanged rather than emitting duplicated or truncated content.
fn replace_region(src: &str, tag: &str, new_body: &str) -> String {
    let open = format!("---- {tag} ----");
    let close = format!("---- END {tag} ----");
    let Some((_, body_start)) = find_marker_line(src, &open) else { return src.to_string() };
    let Some((line_start_rel, _)) = find_marker_line(&src[body_start..], &close) else {
        return src.to_string();
    };
    let line_start = body_start + line_start_rel;
    format!("{}{}{}", &src[..body_start], new_body, &src[line_start..])
}

pub fn splice(existing: &str, reg: &Registry, provenance: &str) -> String {
    let mut out = replace_region(existing, "MANUAL", &body(&reg.manual));
    out = replace_region(&out, "GENERATED", &body(&reg.generated));
    out = replace_region(&out, "RETIRED", &body(&reg.retired));
    rewrite_provenance_line(&out, provenance)
}

/// Rewrite only the `//! Provenance: ...` line, byte-for-byte everywhere
/// else. `lines().join("\n")` would reflow the entire file — silently
/// turning CRLF into LF outside the markers and adding a trailing newline
/// where none existed — so splicing an already-spliced file would not
/// reproduce the same bytes.
fn rewrite_provenance_line(src: &str, provenance: &str) -> String {
    let stamp = format!("//! Provenance: {provenance}");
    let mut offset = 0;
    for piece in src.split_inclusive('\n') {
        let (content, term) = match piece.strip_suffix("\r\n") {
            Some(c) => (c, "\r\n"),
            None => match piece.strip_suffix('\n') {
                Some(c) => (c, "\n"),
                None => (piece, ""),
            },
        };
        if content.starts_with("//! Provenance:") {
            return format!("{}{}{}{}", &src[..offset], stamp, term, &src[offset + piece.len()..]);
        }
        offset += piece.len();
    }
    format!("{stamp}\n{src}")
}

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
        assert_eq!(failures, 118, "on-curve rejection rate changed; the off-curve check may be disabled");
    }

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

    #[test]
    fn parse_entry_captures_trailing_comment_and_splice_round_trips_it() {
        // A hand-written comment on a MANUAL entry must survive a
        // parse-then-splice round trip, or the generator silently deletes
        // human-maintained notes the first time it runs.
        let e = parse_entry(
            r#"PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"), // exchange, verified 2026-01"#,
        )
        .unwrap();
        assert_eq!(e.name, "kraken");
        assert_eq!(e.note.as_deref(), Some("exchange, verified 2026-01"));

        let no_comment = parse_entry(
            r#"PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),"#,
        )
        .unwrap();
        assert_eq!(no_comment.note, None);

        let mut reg = Registry::default();
        reg.manual.push(e);
        let existing = "// ---- MANUAL ----\n// ---- END MANUAL ----\n";
        let out = splice(existing, &reg, "epoch 1028");
        assert!(
            out.contains("// exchange, verified 2026-01"),
            "manual comment must survive splice, got: {out}"
        );
    }

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
    fn returning_pool_is_promoted_out_of_retirement() {
        // A pool that dips below the threshold for one epoch and recovers is a
        // candidate again. Leaving the old entry in RETIRED while also adding a
        // fresh one to GENERATED would duplicate its authority, and
        // parse_registry rejects duplicate authorities — bricking the very next
        // generator run.
        let mut reg = Registry::default();
        reg.retired.push(Entry { name: "phantom".into(), authority: "P1".into(), note: None });

        let out = assign_names(&reg, &[cand("P1", "PoolP", Some("Phantom Staked SOL"))]);

        let matches: Vec<_> = out.all().filter(|e| e.authority == "P1").collect();
        assert_eq!(matches.len(), 1, "authority must appear exactly once across all sections");
        assert_eq!(out.generated.len(), 1);
        assert_eq!(out.generated[0].authority, "P1");
        assert_eq!(out.generated[0].name, "phantom", "returning pool keeps its frozen name");
        assert!(out.retired.is_empty(), "returning pool must be removed from retired");
    }

    #[test]
    fn notes_survive_the_next_run_and_staleness_is_re_derived() {
        // Run N writes `// TODO: name` on a pool with no Sanctum name. On run
        // N+1 that pool is frozen, and the frozen path used to emit it with no
        // note at all — deleting every reviewer annotation on a GENERATED line
        // and making the generator produce a different file on each run.
        let first = assign_names(&Registry::default(), &[cand("A1", "PoolAddr12345", None)]);
        assert_eq!(first.generated[0].note.as_deref(), Some("TODO: name"));

        let second = assign_names(&first, &[cand("A1", "PoolAddr12345", None)]);
        assert_eq!(second.generated, first.generated, "second run must reproduce the first");

        // Staleness is generator-owned: it must appear when the cached balance
        // goes stale on a new entry, not only on the run after.
        let mut stale = cand("A1", "PoolAddr12345", None);
        stale.stale = true;
        let fresh_stale = assign_names(&Registry::default(), &[stale.clone()]);
        assert_eq!(
            fresh_stale.generated[0].note.as_deref(),
            Some("TODO: name; stale: cached balance")
        );

        // ...and clear again once the pool stops being stale, rather than
        // sticking to the entry forever.
        let third = assign_names(&second, &[stale]);
        assert_eq!(third.generated[0].note.as_deref(), Some("TODO: name; stale: cached balance"));
        let fourth = assign_names(&third, &[cand("A1", "PoolAddr12345", None)]);
        assert_eq!(fourth.generated[0].note.as_deref(), Some("TODO: name"));
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

    #[test]
    fn splice_ignores_a_decoy_marker_mention_above_the_real_one() {
        // A doc comment explaining the marker format (which Task 6 is about to
        // hand-write into src/pools.rs) must not be mistaken for the marker
        // itself, or replace_region locates the wrong span and eats everything
        // between the decoy and the real end marker.
        let existing = concat!(
            "//! Sections are delimited by lines like ---- GENERATED ---- and its END pair.\n",
            "pub fn get_pool_by_name(n: &str) -> Option<&PoolInfo> { todo!() }\n",
            "// ---- GENERATED ----\n",
            "        PoolInfo::new(\"old\", \"OldAuth\"),\n",
            "// ---- END GENERATED ----\n",
            "#[cfg(test)] mod tests { }\n",
        );
        let mut reg = Registry::default();
        reg.generated.push(Entry { name: "new".into(), authority: "NewAuth".into(), note: None });

        let out = splice(existing, &reg, "epoch 1028");
        assert!(out.contains("get_pool_by_name"), "accessors must survive the decoy");
        assert!(out.contains("#[cfg(test)] mod tests"), "tests must survive the decoy");
        assert!(out.contains("Sections are delimited by"), "the decoy comment line itself must survive");
        assert!(out.contains("NewAuth") && !out.contains("OldAuth"), "body replaced, not the decoy region");
    }

    #[test]
    fn replace_region_fails_safe_when_end_marker_precedes_start_marker() {
        // With a naive whole-file `find`, an END marker appearing earlier in
        // the file than the real start marker pairs with it anyway, and the
        // span between them gets emitted twice. A missing marker already fails
        // safe by returning the input unchanged; out-of-order must do the same.
        let existing = concat!(
            "// ---- END GENERATED ----\n",
            "        PoolInfo::new(\"old\", \"OldAuth\"),\n",
            "// ---- GENERATED ----\n",
        );
        let out = replace_region(existing, "GENERATED", "REPLACED\n");
        assert_eq!(out, existing, "out-of-order markers must fail safe, not duplicate content");
    }

    #[test]
    fn splice_only_rewrites_the_provenance_line_not_the_whole_file() {
        // lines().join("\n") would silently convert CRLF to LF outside the
        // markers and add a trailing newline where none existed — neither is a
        // byte the provenance rewrite is allowed to touch.
        let existing = concat!(
            "//! Provenance: epoch 1000\r\n",
            "// ---- GENERATED ----\r\n",
            "// ---- END GENERATED ----\r\n",
            "// trailer\r\n",
        );
        let reg = Registry::default();
        let out = splice(existing, &reg, "epoch 1028");
        assert!(out.contains("//! Provenance: epoch 1028"));
        assert!(out.contains("// trailer\r\n"), "CRLF outside the provenance line must survive, got: {out:?}");
    }

    #[test]
    fn splice_is_idempotent_on_its_own_output() {
        // The generator must be idempotent: re-running splice on a file it
        // already produced must reproduce the same bytes, not drift.
        let mut reg = Registry::default();
        reg.generated.push(Entry { name: "a".into(), authority: "Aaa".into(), note: None });
        let existing = "//! Provenance: epoch 1000\n// ---- GENERATED ----\n// ---- END GENERATED ----\n";
        let once = splice(existing, &reg, "epoch 1028");
        let twice = splice(&once, &reg, "epoch 1028");
        assert_eq!(once, twice, "splice must be idempotent on its own output");
    }
}
