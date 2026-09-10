//! Offline pool discovery. Compiled only with the `discover` feature.

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
}
