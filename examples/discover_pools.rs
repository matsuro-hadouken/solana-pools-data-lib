//! Regenerates src/pools.rs from the on-chain stake pool programs.
//!
//! Run: cargo run --features discover --example discover_pools -- --min-sol 1
//!
//! The registry is spliced in place: this program READS src/pools.rs first, to
//! pick up the manual entries and the frozen authority->name bindings, then
//! renames a temp file over it. Never invoke it with a shell redirect —
//! `> src/pools.rs` truncates the file before the process starts and destroys
//! exactly the input it needs.

use serde_json::{json, Value};
use solana_pools_data_lib::discovery::*;
use solana_pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Duration;

const SANCTUM_LIST: &str =
    "https://raw.githubusercontent.com/igneous-labs/sanctum-lst-list/master/sanctum-lst-list.toml";
const STAKE_PROGRAM: &str = "Stake11111111111111111111111111111111111111";

type BoxErr = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxErr> {
    // Every unrecognized or unparseable argument is fatal and names the token.
    // Ignoring them would let `--verfiy` skip the safety gate in silence, and a
    // fat-fingered `--min-sol 1o` fall back to the default — on a tool whose
    // output is permanent public API surface, both deserve a stop.
    let (mut min_sol, mut verify) = (1.0f64, false);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--verify" => verify = true,
            "--min-sol" => {
                let v = args.next().ok_or("--min-sol needs a value")?;
                min_sol = v
                    .parse()
                    .map_err(|_| format!("--min-sol: {v:?} is not a number"))?;
                // NaN would make every `total_sol < min_sol` false and admit the
                // whole chain; a negative threshold does the same.
                if !min_sol.is_finite() || min_sol < 0.0 {
                    return Err(format!("--min-sol: {v:?} is not a usable threshold").into());
                }
            }
            other => {
                return Err(format!(
                    "unrecognized argument {other:?}; expected `--min-sol <SOL>` and/or `--verify`"
                )
                .into())
            }
        }
    }
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let out_path = "src/pools.rs";

    // getProgramAccounts over ~1400 611-byte accounts is a multi-megabyte
    // response; the default client has no timeout at all, so a stalled
    // connection would hang the generator forever instead of failing.
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let epoch = get_epoch(&http, &rpc_url).await?;

    // 1. Enumerate all three programs. A program returning zero accounts means
    //    the layout or program ID assumption broke, not that pools vanished.
    let mut by_program: HashMap<&str, Vec<StakePool>> = HashMap::new();
    for program in [SPL_STAKE_POOL, SANCTUM_SPL, SANCTUM_SPL_MULTI] {
        let pools = fetch_stake_pools(&http, &rpc_url, program).await?;
        if pools.is_empty() {
            return Err(format!(
                "program {program} returned no StakePool accounts; \
                 layout or program ID assumption has broken"
            )
            .into());
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
                Err(e) => {
                    eprintln!("skip {}: {e}", sp.pool);
                    continue;
                }
            };
            all_derived.insert(authority.clone());

            let stale = epoch.saturating_sub(sp.last_update_epoch) > 10;
            let mut total_sol = sp.total_lamports as f64 / 1e9;

            // A stale cached balance may not admit a NEW entry on its own.
            if stale && total_sol >= min_sol {
                total_sol = live_stake_sol(&http, &rpc_url, &authority).await?;
            }
            if total_sol < min_sol {
                continue;
            }

            let (name, symbol) = sanctum
                .get(&sp.mint.to_string())
                .cloned()
                .map(|(n, s)| (Some(n), Some(s)))
                .unwrap_or((None, None));
            cohort.push(Candidate {
                authority,
                pool: sp.pool.to_string(),
                mint: sp.mint.to_string(),
                sanctum_name: name,
                sanctum_symbol: symbol,
                total_sol,
                stale,
            });
        }
        cohorts.insert(program, cohort);
    }

    let existing = std::fs::read_to_string(out_path)?;
    let mut reg = parse_registry(&existing)?;
    // First run: moves the derivable entries out of MANUAL into GENERATED.
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
                if stake_account_count(&http, &rpc_url, &c.authority).await? > 0 {
                    nonempty += 1;
                }
            }
            if nonempty == 0 {
                eprintln!(
                    "ABORT: all {} new authorities under {program} returned no stake \
                     accounts; derivation for this program looks broken",
                    fresh.len()
                );
                std::process::exit(1);
            }
        }
        candidates.extend(cohort);
    }

    // Two pools deriving one authority would make assign_names' _2/_3 suffixes
    // depend on input order — and `cohorts` is a HashMap, so that order is not
    // even stable across runs. Sort, then keep the first of each authority, so
    // the survivor is the same every time. This should never fire.
    candidates.sort_by(|a, b| (&a.authority, &a.pool).cmp(&(&b.authority, &b.pool)));
    let mut first_pool: HashMap<String, String> = HashMap::new();
    candidates.retain(|c| match first_pool.get(&c.authority).cloned() {
        Some(prev) => {
            eprintln!(
                "WARNING: pools {prev} and {} both derive authority {}; keeping {prev}",
                c.pool, c.authority
            );
            false
        }
        None => {
            first_pool.insert(c.authority.clone(), c.pool.clone());
            true
        }
    });

    // 4. Assign names, render, write atomically.
    let provenance = format!(
        "{rpc_url} epoch {epoch}, --min-sol {min_sol}, {} pools",
        candidates.len()
    );
    let updated = assign_names(&reg, &candidates);
    let spliced = splice(&existing, &updated, &provenance);

    // splice() fails safe: a marker line it cannot match exactly leaves that
    // region untouched and hands back the input. parse_registry matches markers
    // with `contains`, so src/pools.rs can parse fine and still splice to a
    // no-op — the generator would then rename an unchanged file into place and
    // report success. Re-parse the output and confirm it holds what we wrote.
    // (Byte-comparing against `existing` cannot be the test: an idempotent
    // re-run legitimately reproduces the same bytes.)
    let round_trip =
        parse_registry(&spliced).map_err(|e| format!("spliced output does not re-parse: {e}"))?;
    if entry_keys(&round_trip) != entry_keys(&updated) {
        return Err(format!(
            "splice wrote nothing usable: {} entries in, {} entries back out. Each \
             section marker in {out_path} must be a line that is exactly \
             `// ---- MANUAL ----` (and GENERATED / RETIRED, plus their \
             `// ---- END X ----` pairs) — replace_region matches them exactly.",
            entry_keys(&updated).len(),
            entry_keys(&round_trip).len()
        )
        .into());
    }

    let tmp = format!("{out_path}.tmp");
    std::fs::write(&tmp, &spliced)?;
    std::fs::rename(&tmp, out_path)?;
    eprintln!(
        "wrote {out_path}: {} manual, {} generated, {} retired",
        updated.manual.len(),
        updated.generated.len(),
        updated.retired.len()
    );
    Ok(())
}

/// Every (authority, name) binding in a registry, order-independent.
fn entry_keys(reg: &Registry) -> Vec<(String, String)> {
    let mut v: Vec<_> = reg.all().map(|e| (e.authority.clone(), e.name.clone())).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------
// RPC
// ---------------------------------------------------------------------------

/// Space calls out. The public endpoint 429s a full run otherwise, and the
/// answer to a rate limit is to go slower, not to enumerate fewer pools. Set
/// DISCOVER_RPC_DELAY_MS=0 when SOLANA_RPC_URL points at a private node.
fn pace() -> Duration {
    Duration::from_millis(
        std::env::var("DISCOVER_RPC_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100),
    )
}

/// One JSON-RPC round trip. 429 and 5xx are retried with a doubling backoff;
/// a JSON-RPC `error` object is fatal, because it means the request itself was
/// wrong and retrying only asks the same wrong question again.
async fn rpc(
    http: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
) -> Result<Value, BoxErr> {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    let mut backoff = Duration::from_millis(500);
    for _ in 0..6 {
        tokio::time::sleep(pace()).await;
        let resp = http.post(url).json(&body).send().await?;
        let status = resp.status();
        if status.as_u16() == 429 || status.is_server_error() {
            tokio::time::sleep(backoff).await;
            backoff *= 2;
            continue;
        }
        let v: Value = resp.error_for_status()?.json().await?;
        if let Some(e) = v.get("error") {
            return Err(format!("{method}: RPC error {e}").into());
        }
        return v
            .get("result")
            .cloned()
            .ok_or_else(|| format!("{method}: response carried no result").into());
    }
    Err(format!("{method}: still rate limited after 6 attempts; raise DISCOVER_RPC_DELAY_MS").into())
}

async fn get_epoch(http: &reqwest::Client, url: &str) -> Result<u64, BoxErr> {
    rpc(http, url, "getEpochInfo", json!([])).await?["epoch"]
        .as_u64()
        .ok_or_else(|| "getEpochInfo: no epoch in result".into())
}

/// Every StakePool account under `program`. The dataSize filter pins the
/// 611-byte layout and the memcmp pins byte 0 to AccountType::StakePool
/// (base58 "2" == 0x01), so ValidatorList accounts never reach `decode`.
async fn fetch_stake_pools(
    http: &reqwest::Client,
    url: &str,
    program: &str,
) -> Result<Vec<StakePool>, BoxErr> {
    let result = rpc(
        http,
        url,
        "getProgramAccounts",
        json!([program, {
            "encoding": "base64",
            "filters": [
                {"dataSize": STAKE_POOL_LEN},
                {"memcmp": {"offset": 0, "bytes": "2", "encoding": "base58"}}
            ]
        }]),
    )
    .await?;

    let accounts = result
        .as_array()
        .ok_or_else(|| format!("getProgramAccounts({program}): result is not an array"))?;
    let mut out = Vec::with_capacity(accounts.len());
    for acc in accounts {
        let pubkey = acc["pubkey"]
            .as_str()
            .ok_or("getProgramAccounts: account without pubkey")?;
        let encoded = acc["account"]["data"][0]
            .as_str()
            .ok_or("getProgramAccounts: account without base64 data")?;
        match decode(Pubkey::from_str(pubkey)?, &b64(encoded)?) {
            Ok(sp) => out.push(sp),
            // One odd account is worth naming, not worth aborting the run for.
            // A systematic break shows up as an empty cohort, which main aborts on.
            Err(e) => eprintln!("skip {pubkey}: {e}"),
        }
    }
    Ok(out)
}

/// Every stake account whose staker (offset 12) is `authority`, carrying account
/// metadata only — the zero-length dataSlice transfers no account data, which
/// keeps a several-thousand-account response small.
///
/// Both callers share this one request shape on purpose. They used to build
/// their own, and the pair silently drifted into measuring different things.
async fn stake_accounts(
    http: &reqwest::Client,
    url: &str,
    authority: &str,
) -> Result<Vec<Value>, BoxErr> {
    let result = rpc(
        http,
        url,
        "getProgramAccounts",
        json!([STAKE_PROGRAM, {
            "encoding": "base64",
            "dataSlice": {"offset": 0, "length": 0},
            "filters": [{"memcmp": {"offset": 12, "bytes": authority, "encoding": "base58"}}]
        }]),
    )
    .await?;
    match result {
        Value::Array(v) => Ok(v),
        _ => Err("getProgramAccounts(stake): result is not an array".into()),
    }
}

/// How many stake accounts `authority` stakes. Zero across a whole cohort is
/// what `--verify` treats as a broken derivation.
async fn stake_account_count(
    http: &reqwest::Client,
    url: &str,
    authority: &str,
) -> Result<usize, BoxErr> {
    Ok(stake_accounts(http, url, authority).await?.len())
}

/// Live balance behind `authority`, in SOL: the sum of `account.lamports` over
/// every stake account it stakes.
///
/// This must measure the same quantity as the non-stale path, which compares
/// against `StakePool.total_lamports`. Summing `Stake.delegation.stake` at
/// offset 156 does not: it counts delegated stake only, so it misses the
/// reserve account, transient stake and anything undelegated — and it reads
/// zero for the reserve, which is a stake account in the Initialized state with
/// no delegation at all. A pool holding everything in reserve is legitimate —
/// the case Task 7 exists to support — so judging a stale pool that way applies
/// a stricter rule than a fresh one gets, and drops pools that belong in the
/// registry. It dropped 26 of them before this was corrected.
///
/// `lamports` lives in the account metadata, not in `data`, so the zero-length
/// slice costs nothing and still answers the question.
async fn live_stake_sol(http: &reqwest::Client, url: &str, authority: &str) -> Result<f64, BoxErr> {
    let mut lamports: u64 = 0;
    for acc in stake_accounts(http, url, authority).await? {
        // A missing `lamports` would otherwise read as a zero balance and
        // silently retire a live pool.
        lamports = lamports.saturating_add(
            acc["account"]["lamports"]
                .as_u64()
                .ok_or("stake account without lamports")?,
        );
    }
    Ok(lamports as f64 / 1e9)
}

/// mint -> (name, symbol) from Sanctum's LST list. This is the only source of
/// human names; without it every new pool would land as `unnamed_*`, so an
/// empty parse is an error rather than a silent downgrade.
async fn fetch_sanctum(http: &reqwest::Client) -> Result<HashMap<String, (String, String)>, BoxErr> {
    let text = http
        .get(SANCTUM_LIST)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let doc: toml::Value = text.parse()?;
    let list = doc
        .get("sanctum_lst_list")
        .and_then(|v| v.as_array())
        .ok_or("sanctum-lst-list.toml has no sanctum_lst_list array")?;
    let map: HashMap<String, (String, String)> = list
        .iter()
        .filter_map(|e| {
            Some((
                e.get("mint")?.as_str()?.to_string(),
                (
                    e.get("name")?.as_str()?.to_string(),
                    e.get("symbol")?.as_str()?.to_string(),
                ),
            ))
        })
        .collect();
    if map.is_empty() {
        return Err("sanctum-lst-list.toml parsed to zero usable entries".into());
    }
    eprintln!("sanctum-lst-list: {} named mints", map.len());
    Ok(map)
}

/// Base64 decode. The RPC returns account data base64-encoded and the crate has
/// no base64 dependency; twenty lines beats adding one to the dependency tree of
/// a library whose default build must stay lean.
fn b64(s: &str) -> Result<Vec<u8>, BoxErr> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for c in s.bytes() {
        let v: u32 = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => return Err(format!("invalid base64 byte {:?}", c as char).into()),
        }
        .into();
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::b64;

    #[test]
    fn decodes_base64_including_padding_and_rejects_garbage() {
        assert_eq!(b64("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(b64("AQID").unwrap(), vec![1u8, 2, 3]);
        assert_eq!(b64("AQ==").unwrap(), vec![1u8]);
        assert!(b64("!!!!").is_err());
    }
}
