//! Pool authority definitions and utilities.
//!
//! This module contains the embedded list of known stake pool authorities
//! and provides utilities for working with pool information.
//! Provenance: https://api.mainnet-beta.solana.com slot 447659744 epoch 1036, --min-sol 1, --unnamed-min-sol 5000, sanctum-lst-list d0beb503dca1d1c6380386da47a79a259324f436b0d6a8e65dcbbc5dcd7ad74b, 238 pools

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Information about a stake pool
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolInfo {
    /// Human-readable name of the pool
    pub name: String,
    /// Base58-encoded authority public key
    pub authority: String,
}

impl PoolInfo {
    /// Create a new `PoolInfo`
    pub fn new(name: impl Into<String>, authority: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            authority: authority.into(),
        }
    }
}

// ===========================================================================
// The pool lists below are GENERATED. Do not hand-edit them.
//
//   cargo run --features discover --example discover_pools -- --min-sol 1
//
// This block of guidance lives OUTSIDE the section markers on purpose: the
// generator replaces every byte between an opening marker line and its closing
// pair, so anything written inside a marked region is erased on the next run.
// (It also spells the markers below with a `NAME` placeholder rather than their
// real text. That is belt-and-braces, not a requirement: both the parser and the
// splicer match markers exactly, so a mention inside a sentence is already not a
// marker. Keeping the placeholder means a human skimming this file can still
// tell guidance from a real marker at a glance — and a future regression back to
// loose matching would not silently turn this paragraph into one.)
//
// Rules the generator relies on:
//
//   * `#[rustfmt::skip]` on both statics is load-bearing, not cosmetic.
//     Without it `cargo fmt` rewraps the ~121 `PoolInfo::new(..)` calls that
//     exceed 100 columns onto four lines; the generator's parser requires the
//     whole call on one line, so those entries would be rejected (loudly, now)
//     and, before that check existed, silently dropped — losing the frozen
//     authority->name bindings that ARE this crate's public API.
//   * A section marker must be a line that is EXACTLY `// ---- NAME ----`,
//     where NAME is MANUAL, GENERATED or RETIRED, each closed by an equally
//     exact `// ---- END NAME ----`. The parser that reads this file and the
//     splicer that rewrites it apply that same test, so they cannot disagree
//     about where a section starts and ends. A marker embedded in a longer
//     sentence is not matched, and the region it was meant to open is left
//     untouched.
//   * Inside the markers, only one-line `PoolInfo::new("name", "authority"),`
//     entries, blank lines, and `//` comments are legal. Anything else aborts
//     the generator. A trailing `// note` on an entry, or a standalone comment,
//     may say anything at all — including marker text; neither is a marker.
//   * The MANUAL section is hand-curated in content only. The generator
//     re-renders it and re-sorts it by authority on every run; trailing
//     `// notes` are carried across, arbitrary non-entry code is not.
//     It also MOVES entries out: a manual entry whose authority turns out to be
//     a derivable SPL-family pool is promoted into GENERATED, and if that pool
//     sits below --min-sol it is retired in the same run — so it leaves
//     get_active_pools() and fetch_all_pools(). Its name and note survive, so no
//     API key breaks, but a hand-added pool under the threshold will not stay in
//     the active set.
//   * A pool that no upstream source names is WITHHELD rather than given a
//     placeholder, unless it holds at least --unnamed-min-sol. A withheld pool
//     is not retired: it was never in the registry, and it appears the moment it
//     is named or grows. A pool already here is never withheld, whatever
//     upstream does, because its name is already frozen.
//   * A pool name, once emitted, is frozen forever — it is a public API key.
//     Identity is the authority pubkey; the name is only a label.
// ===========================================================================

/// Static registry of all known pools, split into active and retired.
#[rustfmt::skip]
static POOLS_ACTIVE: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    vec![
        // ---- MANUAL ----
        PoolInfo::new("galaxy", "2Wk4x9xaxmqvXr7amKA36UaQzPcqri3HCGmg6wWxo7c5"),
        PoolInfo::new("kraken", "36kaqVpcbSSJ55rP48uGQWtQs3eNaa6SbX8qbhPxHGJf"),
        PoolInfo::new("kiln_2", "3F7m8xoKv96HRuLj3okKeR9TGTocWe5M43bjYeDeXNaW"),
        PoolInfo::new("galaxy_3", "3j79gPSKJReLX5pn8D6CDwViJ9kFCV1Es27X882DgLXk"),
        PoolInfo::new("sanctum_2", "3rBnnH9TTgd3xwu48rnzGsaQkSr1hR64nY71DrDt6VrQ"),
        PoolInfo::new("marinade", "4bZ6o3eUUNXhKuqjdCnCoPAoLgWiuLYixKaxoa8PpiKk"),
        PoolInfo::new("forward_industries_2", "7d4ZhfBRamc2szcuHbVGYbuKFNfjZoKeXm2S3JC2uXeP"),
        PoolInfo::new("kraken_4", "7znb4m2EahgFD29DQ3m285UnbE7die5945BGUm6rHLP8"),
        PoolInfo::new("upbit", "8XmdpcJwcqyoezXDbnmMRoxtdfAqkbSxfZbyvXwKnn76"),
        PoolInfo::new("binance", "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
        PoolInfo::new("figment", "9vL28oEG95zcqAe2DRdA5nmv6jLiRbh8jj9Qbd9gxrH"),
        PoolInfo::new("jupiter_2", "AVzP2GeRmqGphJsMxWoqjpUifPpCret7LqWhD8NWQK49"),
        PoolInfo::new("kraken_2", "AeR3Jpkf9AroKyzFEuvxmxq2AJdoQ59qiFFVkzUTcfxX"),
        PoolInfo::new("p2p", "AxqtG9SHDkZTLSWg81Sp7VqAzQpRqXtR9ziJ3VQAS8As"),
        PoolInfo::new("socean", "AzZRvyyMHBm8EHEksWxq4ozFL7JxLMydCDMGhqM6BVck"),
        PoolInfo::new("twinstake_2", "B1FN6n7pzuKyFnnKC68jU9UePTdAFqhcBMMsYJ5UeyYk"),
        PoolInfo::new("figment_2", "BdZBQGZ3icyTGyhmQnzTpnWmQjDmHbGpmenWN6QPNipE"),
        PoolInfo::new("eversol", "C4NeuptywfXuyWB9A7H7g5jHVDE8L6Nj2hS53tA71KPn"),
        PoolInfo::new("figment_4", "CCyieXV1E2Pyi2UssnDaGykcTDjfw1aK8MJYiyqKPkTb"),
        PoolInfo::new("kraken_3", "CGYY3tmbX671yGmJqSSkLpyDcMTt6RCrve6KH28fiay1"),
        PoolInfo::new("kiln", "CyzVd8xyNjUsnPu3ra5fDPW2vja8zXpVbMoEk8hU5n9g"),
        PoolInfo::new("galaxy_2", "DEGv8tV9RBEULiNJC84UuTFWJZeFyLrmShpBiV5ubenp"),
        PoolInfo::new("twinstake", "Dc7VoDch4LifBnLypYKbaJCgBMHSncoP2dorEeuEHctY"),
        PoolInfo::new("firedancer_delegation", "FiRep26iRQbMaKbqhhs5CqXqy7YrHn462LbnQhXzB2ps"),
        PoolInfo::new("figment_3", "G2NEckNz42pWoFkb4KLuvhLcepMyXN4Coy7rM5dHM1bK"),
        PoolInfo::new("okx", "GcJFx1MZJ8Zn7PwtRprnzrXz4NyApdm16yAkCFQ4JvG"),
        PoolInfo::new("ftx", "H4yiPhdSsmSMJTznXzmZvdqWuhxDRzzkoQMEWXZ6agFZ"),
        PoolInfo::new("marinade_select", "STNi1NHDUi6Hvibvonawgze8fM83PFLeJhuGMEXyGps"),
        PoolInfo::new("lido", "W1ZQRwUfSkDKy2oefRBUWph82Vr2zg9txWMA8RQazN5"),
        PoolInfo::new("binance_3", "c5f9zfpkKMD9N8uLqJcFeJAAz7v12vDMnup9Y6EeQkk"),
        PoolInfo::new("marinade_native", "ex9CfkBZZd6Nv9XdnoDmmB45ymbu4arXVk7g5pWnt3N"),
        PoolInfo::new("foundation", "mpa4abUkjQoAvPzREkh5Mo75hZhPFQ2FSH6w7dWKuQ5"),
        PoolInfo::new("marinade_native_2", "stWirqFCf2Uts1JBL1Jsd3r6VBWhgnpdPxCTe1MFjrq"),
        // ---- END MANUAL ----

        // ---- GENERATED ----
        PoolInfo::new("shark_pool", "12bX3M9rnu1HWG87BwGfxeE5ouhWAJpdSqwBWiP8hnuQ"), // stale: cached balance >10 epochs old
        PoolInfo::new("hylo", "2C9aTiNL6VyrPhFKspZC8BY9JeL3j4RtkPP2e4PrVAwP"),
        PoolInfo::new("science", "2CJbnuRygrzVYoKbLhJbwq16x46ERARFPzBe57pyuiGg"),
        PoolInfo::new("theta", "2SqLr2wcXDDMSozGNaDVo1L1t781WvVjyt4v4hwAuqrx"),
        PoolInfo::new("soul", "2bKmVmyqCw4LFa2sWeKpJH7Leq7MjTzfWDdWogZHPjey"), // verify: from "soulSOL"
        PoolInfo::new("exchange_art", "2n3MKK5X4v7V6SZ9fXNBouhpRpDYxhPpf3FktAbxSm7Q"),
        PoolInfo::new("pathfinders", "2oFvntttTFoEejX5uRoP5NLovNjhe7fiCZhs3Td1tpYa"),
        PoolInfo::new("helius", "2rMuGTyXCqCHZBSu6NZR9Aq8MhZX9gLkCHoQsPhSj2YF"),
        PoolInfo::new("banx", "2wTD2CJN3VSfV84JUexQLmhUeCbXPjKBYKFME3LWt4ue"), // verify: from "banxSOL"
        PoolInfo::new("phantom", "2zeXA69dFoMLFLdzCfcff35aL7Y7uBZnjKBGWtESgVfS"),
        PoolInfo::new("sphere", "39jEZe3EqDn23vZbGuGzL6NnkmUZiox5uS7Y1ZvAv6z8"),
        PoolInfo::new("superfast", "3DNHyFDogmurGQsrj6g93sm9LSbenzRyF6n97osCnJi1"),
        PoolInfo::new("famous_fox_federation", "3KVYY9bckvHTnYizVYZbra3wf6XnWpugNXvRmbxGAwoN"),
        PoolInfo::new("apy", "3LXLNjGGS8XhPmco5vhScyW8cb3TVPmpF7pr7k7ZfCoW"), // verify: from "apySOL"
        PoolInfo::new("meme", "3LoW79HWTp5MvGSUkDcBWggzhwGRteeNbLRj7nwrAVKN"),
        PoolInfo::new("power", "3SWDH9uUc9Vt45Nb6WbnEKWePGigz6hErDgQES6tdQ8Z"),
        PoolInfo::new("compass", "3SpAsJj9mXsDmwtaE6zSgEh78ZZE278TBnYgAAy5DHaM"),
        PoolInfo::new("gripto", "3SyZ82mdjy8QwgVTBc1PDPcsWnH1gqgsLqJdsaaAJ4H9"),
        PoolInfo::new("sana", "3VL2UAv1BkAshXzTghALgQdxVbgYoNh5Kozk7xRL7QnB"), // verify: from "SanaSOL"
        PoolInfo::new("genesis_toby", "3Z1TUgrD9BQ8abwCTgTD6AZVsQDFuY1Uchy5N6tNPbeN"),
        PoolInfo::new("marginfi", "3b7XQeZ8nSMyjcQGTFJS5kBw4pXS2SqtB9ooHCnF2xV9"),
        PoolInfo::new("forward_industries", "3ndMuPC9Cz5VC4RJkpoPaZz6Px6eVXtRenw9Yi1o2xnA"),
        PoolInfo::new("bybit", "3pFTQjRVwcJHSpUNH5n1hx6Jwx7V3EzJDDHaKuwExyGJ"),
        PoolInfo::new("picosol", "4At8nQXanWgRvjbrVXmxMBBdfz39txWVm4SiXEoP1kGh"),
        PoolInfo::new("lstache", "4FRxJaH6P5Z5mTMkGXezPdpUJPBA6cWrZY8ihfhaCbXw"),
        PoolInfo::new("save", "4HBJeDeg2oLVsk6GButjc4xffWajbwTv2v7Uwi7DvCs1"), // stale: cached balance >10 epochs old
        PoolInfo::new("camao", "4J1MBDZo48T9MMvyzijVnjVTGqfkhXnEhbEseHaLkipq"), // verify: from "camaoSOL"
        PoolInfo::new("infinite_lux", "4R1L9q6sRopHKtQD1rpd7tBnUcu1jduXD3C6LfX3ViaV"),
        PoolInfo::new("project_super", "4SdQDBVsdS6CkfZrBMi9woJPACF62C3PBrkZ6UfiaqDt"),
        PoolInfo::new("envbest", "4WbEJTnREi2b3vN4dLriCtZePN8SSW5Dq46G6WiTcji3"),
        PoolInfo::new("double_zero", "4cpnpiwgBfUgELVwNYiecwGti45YHSH3R72CPkFTiwJt"),
        PoolInfo::new("yield", "4ds8188KcEpxVmr6KT1wbtR4SVFG4eKSB8BjmaQcTUFv"), // verify: from "Yield SOL"
        PoolInfo::new("puff", "4hbYiDXmCJXSCRPSENGG7grV4GKZRBpNn1HndBqpMMSk"), // verify: from "puffSOL"
        PoolInfo::new("steakstache", "4mDGwGiohbHSW9qppbU3MsUd6i3RQqxoGseBP7m3YT4h"),
        PoolInfo::new("moonpay", "4qi96eZUEEmHS9caLK54sL5LU15LicRAbR1mQ77DEbLt"),
        PoolInfo::new("pine_stake", "4rcNs6cvpVFSU8iPT1JKzK81PXq82vffFwuuETQHJdgr"),
        PoolInfo::new("urban", "4wNpvPYAgiJoFkXawtyfnnEecK8FF932J8YFraj4hzTu"), // stale: cached balance >10 epochs old
        PoolInfo::new("pumpkin_s", "51BsDVZKfFSphgKReCJyEHRVkrSj9827PX3L9KmryJWA"),
        PoolInfo::new("ice", "57RrmRqAyha3jT6Z3RZom6EfwRm1q61Rw4Fk9hj4345E"), // verify: from "iceSOL"
        PoolInfo::new("reflect", "5BKPEg79DJytryo7tUvBm8s6zjv5wSpVxTuSd8Xd3M83"),
        PoolInfo::new("vybe", "5Bsdsw84hcDhCssMnNSUjWx3EqvfD3dV8AX2BGSRuMZa"),
        PoolInfo::new("katexbt", "5CVW3Ao9PjcW5NwLims7L5fG3JJz8e8td9iyiJNNhVJj"),
        PoolInfo::new("uwu_2", "5FcHFpFuXyX4jW7jW1PLLzsqiqpZ6a1tUwjpsFQ4haRd"), // verify: from "uwuSOL"; stale: cached balance >10 epochs old
        PoolInfo::new("yonta_labs", "5M1L2SzLNX6EjXsix8zNfZ4agnk3YoimkXWMndejmet5"),
        PoolInfo::new("digital", "5PRaMuxEBsRr579yDZYcxsds8Lc1WSJD8ugzx6iP3dK"),
        PoolInfo::new("sol_on_bags", "5RjeAvB5wGMS1jZ94Y3nLTa7YRbmsi1G32DRr6chjXiK"),
        PoolInfo::new("rakurai", "5U6ar3a2CQMRhu5d5xBA9E7Gi6xqx45wrXx38vKjU3L9"),
        PoolInfo::new("joetakayama", "5VofEUFou1NUWc5V7jM7ZgNAG7mcDyVhoxF9zpTih6X5"),
        PoolInfo::new("accretion", "5ZCEh3FVpR6yThtZ9o9ieWR6NipFT4xKUMGPuUMCkcCD"),
        PoolInfo::new("mallow_2", "5bQDcgmJv89yTkJBh8aThjnbTjcP6tnTqQkqrb6avFai"), // stale: cached balance >10 epochs old
        PoolInfo::new("stake_city", "5csNBSTkMZTTkJfn9GdRPcSdM3UgoYvgrd7HPdAppDKS"),
        PoolInfo::new("hedgehog_spiky", "5dtGPn1NrSNF5PfHQ1D79GCFjd8ERuN338RktSm5FMAP"),
        PoolInfo::new("backpack", "5hhYv4b1Bt5sdMGYyyvpciwRbyUD1ZWeCmTaQcuvb7Eg"),
        PoolInfo::new("cudis", "5kpFGoqZFhrTho93mgAQrMgkdSB4ruaLxHga5M6FcZPr"),
        PoolInfo::new("sanctum_3", "5rxZiXpLsiuuyFpAocu5dbxmM6HUpygJ6aChmfr5s8Av"),
        PoolInfo::new("nodz", "5ssDdLYeFsuCaAjkDVXoJefUvytwgv23angFioyH8dCP"),
        PoolInfo::new("xandeum", "5uJR4QjnRPzHnt4R2FogtpbzQaUE2HKo3Z9yEYxPTi1H"),
        PoolInfo::new("definity", "5ugu8RogBq5ZdfGt4hKxKotRBkndiV1ndsqWCf7PBmST"),
        PoolInfo::new("STKE", "5vzKiHVuZNx1XQWQZQEcuqKaq4nfDp6LhuSvowQK2ayd"),
        PoolInfo::new("death", "5yFALrVMqZUX9ft2Ph2WzsZKk2bm7UnYpJsmGyF8rzpG"), // stale: cached balance >10 epochs old
        PoolInfo::new("cogent", "5zycDbpcwkm2kqJLVnCqAMRQRSPtNrNLhcncyyZmWR4z"),
        PoolInfo::new("burndao", "66EXYBt7oPsx4rXXHHzxBQxwWehaAEDdZnwaQu7uR2sY"),
        PoolInfo::new("drift", "6727ZvQ2YEz8jky1Z9fqDFG5mYuAvC9G34o2MxwzmrUK"),
        PoolInfo::new("test_lst", "6JoSBoEfq7ivq6Wo4bJnUC4ASKCh3msWnTYAwc5cjD2E"), // stale: cached balance >10 epochs old
        PoolInfo::new("agio", "6LtRu9eXwuUHeYXgBb65q6qYJM21U5FB1BYXdMWQyhPj"),
        PoolInfo::new("sanctum_automated_as4i8", "6PKX9giWtPAdRyhiqMsQGUhTr8t1LewyTmf7KUfpz1aG"),
        PoolInfo::new("aep_life_form", "6Ri5EDyVUmzwePnsJYXVKP7oquDoViidiWodyyqagoZg"),
        PoolInfo::new("lantern", "6Sw4WcMTakZFrd19Q4hTH8ewiLxXXTCuBdVxAaneg1fo"),
        PoolInfo::new("blazestake", "6WecYymEARvjG5ZyqkrVQ6YkhPfujNzWpSPwNKXHCbV2"),
        PoolInfo::new("free_stacc_fun", "6WxWeTJEVFCorfN8irq5grPX2AkjdEsm1gmgFXNR3cnu"), // stale: cached balance >10 epochs old
        PoolInfo::new("magnetic", "6eXiw7sVCMFcHT9cCqRQwxwJVVGvSzWBeqQN7gWTK3Ee"),
        PoolInfo::new("lotus", "6iKy6m3xu2ACMAzxWW66bPbgb71D1aPH46FbdAhV8io8"), // verify: from "lotusSOL"
        PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        PoolInfo::new("elagabalx", "6oxJvKCqAtXFCvZ7bES75JjpqARWX7fEQsv2CABdqvby"),
        PoolInfo::new("vorld", "71c664JxYv7SZ1PatgA2amV1ZNBHhEdP4RXne9KtvU1a"), // verify: from "vorldSOL"
        PoolInfo::new("binance_2", "75NPzpxoh8sXGuSENFMREidq6FMzEx4g2AfcBEB6qjCV"),
        PoolInfo::new("fragmetric", "77Hj6qqjZpqBSLRdJyfk159K9bF5sAyDEFd62GjPqniK"),
        PoolInfo::new("intel", "7F7CwNxpsjk46bNrjpw9hveMHKZc1kT46Hr3icGJF7g8"), // verify: from "intelSOL"
        PoolInfo::new("danger", "7HiwmgWqzynCUG69JE4HmfWZZYtV5KBYjeEEQSVJQvHz"), // stale: cached balance >10 epochs old
        PoolInfo::new("blockponics", "7MVEkencEzNrm6EU6YeTwYCx4toFts8baQvUZCSdoNMA"),
        PoolInfo::new("hydex", "7NdW6sU68yRCUNNuuDF2G9fQJXVkJPnCydyxPQh8dx1Y"),
        PoolInfo::new("dogwif", "7STPYcDtmhgsg2XbyB7PwnNXwWkWKxc8gzycHmT32PXr"),
        PoolInfo::new("wen", "7bfnwncXe7R34K7BSggjYEA6kCuwvpjjtKuCY4hkhpD2"),
        PoolInfo::new("defidevcorp", "7cWNhDsHe1m36ttDkJVBgbee1hRFvPqWZT2iWaBAyYGW"),
        PoolInfo::new("polar", "7cbsCAuHagZq7mx3EmNpQWSoxkCjGfxdGd7d19DntRVb"), // verify: from "polarSOL"
        PoolInfo::new("orca", "7zo3q61Sefdg3X5JLgVnYM48D7qMUhYMPmQkC8yhbSTE"),
        PoolInfo::new("health", "84WcERTbzhHscRWWsm7Xx5vSUQn19nLmQKcWm5VEpnH4"),
        PoolInfo::new("mexc_staked_test1", "8EWtZwo2L2rnNLCtfmYicGzFGDnj1jP48UcG5H42Z75D"),
        PoolInfo::new("solana_id", "8WXgf6CpKNq1RS3Hy1mpGrMYdAT798kC2DReTPTZLzfH"),
        PoolInfo::new("mexc", "8X3sExDr4UcWsyV5Mm7Sg81oup6GxaiLAoLsrcFREa2N"),
        PoolInfo::new("bonk_2", "8bqKkyYd33mKcVwVgrcpQpP3Eogp4BprZRBL8krGPTsi"), // verify: from "bonkSOL"; stale: cached balance >10 epochs old
        PoolInfo::new("stakr_space", "8hYPhXDCDczXLhWZ6i5WNUjmS5kav51eV4RZ2EpseG5V"),
        PoolInfo::new("1st_retard_degen", "8mJE6mDLCaGuo3SyoL2mt4ZVr695ELP2qcUDhrzg3mmL"), // stale: cached balance >10 epochs old
        PoolInfo::new("magic_eden", "8qKdoNUFqcsWuKxiRPjjSo4S467N3WBi7PuM844KnfVB"),
        PoolInfo::new("the_chimpions", "8r1we5gARtq96VJ6zuSEpdgZ9Y1Jqw6wQFQtj78VJFoP"),
        PoolInfo::new("coleta", "8tR9T5FY9qAeBhYkgLwnUdpsnKS3yCtgA5YhE4QS6q7v"),
        PoolInfo::new("starke", "8uKJ5wLUFXYdqTJw2JHRzdcWB8JLR96mEf9dreqD6d5X"),
        PoolInfo::new("luminal", "8uTA5Jn4UxPPVXm9bYziPYrdjnnRLZMmXybmSNQDhDzy"),
        PoolInfo::new("truth", "8vqdF13hrdAy4kN2XSTbwXHQx8rEstLhcx4eAfGMhUjd"), // verify: from "truthSOL"
        PoolInfo::new("step", "8wU5ThsUmjUx4hnDQb3wHswdXenc9GiSJ8ivTpRDaSAz"),
        PoolInfo::new("unstk", "92njzPrnqXq5F75Y5hMYV9jBhPXEzXL2x57AZm6sWVu2"),
        PoolInfo::new("hylo_sol_plus", "92rS1uTEmcATAjap6hW3M34jbNt67kK214PiSkbn25uK"),
        PoolInfo::new("nansen", "92xZLN9vyki6jrqaLk9hEZa4vQsaGBtSRahS3yYJXxeG"),
        PoolInfo::new("danger_2", "9ATyBkLwhk48riNDzG1ka54ksgU2Tdt7bZ8aJygDK9ni"), // stale: cached balance >10 epochs old
        PoolInfo::new("gate_io", "9BadxcrgcZ1in6CfVMsU6PHU45rzLrEjcd4FQMjXNbM5"),
        PoolInfo::new("ecotoken_stake_group", "9BxPrUpYi98t4UWvreocmrc7KtBcBWaFDgeBH5zXhP4j"),
        PoolInfo::new("t_stake", "9EB9Z4YgQZn2F7NaF9iSHBUySTeMbSXQjmpVZBt8xr6c"),
        PoolInfo::new("phase_labs", "9HYjrCF5evTobLmay372kpUWSNTrtmDU2NkQtqjdnHKa"),
        PoolInfo::new("charity", "9JtYqCodjobMtsBUozV9LbB6eyjusuXi4JGL2qgg9nJK"),
        PoolInfo::new("met_lp_army", "9Jv3FdxEHXeHzUYNZEc8KvQGcH8SF1zAnhNfJaChuoob"),
        PoolInfo::new("bonk", "9LcmMfufi8YUcx83RALwF9Y9BPWZ7SqGy4D9VLe2nhhA"),
        PoolInfo::new("uprock", "9X24PaZEb7oR5VDupKLTyMm9YWP1UmptCDfmgNH9NVZ8"),
        PoolInfo::new("japan", "9YRhD7Nz5hcr1aFJ361HNeh5EFEUUEtz2xTHMvRLW71Z"), // verify: from "JapanSOL"
        PoolInfo::new("lifinity", "9Ydzb2u5GUnwWHw1xwaVNZcZeaUrqHe6LmeisbML1KQ5"),
        PoolInfo::new("defi_station", "9r3GQeRydDb4Zv2K6zXGWY5ikjwSV5krE89hAAEnmRBC"),
        PoolInfo::new("ender", "9v9PwBGVnPxMDTrJqEEd5gQc13tgmnDHybcqVDqoE7fm"), // verify: from "enderSOL"; stale: cached balance >10 epochs old
        PoolInfo::new("pine", "9wdjKcLu4V1VNzYytnSdfub4CGRBhKpM4aZpGUdcAgxv"),
        PoolInfo::new("save_2", "9yWcz4S27nXKpsVmWqaimphCUnFo441JUvwkzmvRWys3"),
        PoolInfo::new("danger_3", "A2xeWAFZUMA8ZUskS6SBR5dFAoDjHrA4ATYgheHE2izB"), // stale: cached balance >10 epochs old
        PoolInfo::new("portugal", "A8vgVnLhuxANkxkWuk8665YsW9XEjs2vSqZNuUZUGWb"),
        PoolInfo::new("laine_stake_token", "AAbVVaokj2VSZCmSU5Uzmxi6mxrG1n6StW9mnaWwN6cv"),
        PoolInfo::new("stronghold_lst", "ABCTazdzA7j6CHbnuetnWuvpeKh2XmxqVP75D8KJfsSK"),
        PoolInfo::new("inshallah", "AHk6JWuLCdE5HaXd43LEzMUg7t5WbX3V9ti359rgYiSd"),
        PoolInfo::new("aerosol", "AKJt3m2xJ6ANda9adBGqb5BMrheKJSwxyCfYkLuZNmjn"),
        PoolInfo::new("honest", "AchyTjXJwkFHtX2SSuC3PYc1DwPst3LKXXSHdeqAMMwu"),
        PoolInfo::new("clash", "Agk6TjB4y4yfA531P4eyauEztB46PzpyFVqkZzAWCsTV"),
        PoolInfo::new("eon", "Ahhfr8dZ9hcBG9FX6295tQ4WPqHssyY1nykwLahUNnbu"), // verify: from "eonSOL"
        PoolInfo::new("solanahub", "Am87irh66UBgzkQUrw2XdUYE6r2XLFfADgmSFdZk2K4r"),
        PoolInfo::new("stakehaus", "AqynZHwE3qEcRho7Uu74c3YyW1nxyk7jMoBVevvGWv6Q"),
        PoolInfo::new("rad", "AroCaG2uDCfsKiggRWXCN2CDcKNwgrtZGcpDcLRyd3ed"), // verify: from "radSOL"
        PoolInfo::new("greythorn", "AvBDRcpBPXwQ7Nh2wdJxhbCYKzrpUofhJ8pibC15RXoS"),
        PoolInfo::new("nordic", "AxH3gxpEsXku8n2o4mYQX7obifRUgUWrTyrsc7Abmg32"),
        PoolInfo::new("dain", "AxZbpxUvfVcgT9V5AyvMbZGMicJD6rHmcKfaq3iLCkTn"), // verify: from "dainSOL"
        PoolInfo::new("white_swan", "B47uK6nXjs99mTy6c4oYcspRdpxhFnHnmxsdrs8yNRcD"),
        PoolInfo::new("1st_degen", "B4rqsXDBSFH64L6fk3A9mWeU8ip8Te6se1uURpLmNSew"), // stale: cached balance >10 epochs old
        PoolInfo::new("hanabi", "B7oQQtF7Wdzp7acSqpp5KWWigpywipBN84NVeosf7CxA"),
        PoolInfo::new("thugbirdz", "BAUL1gQ6YkM8eoC7VXyHwbyHGproLJZpSpbo6wywmYe"),
        PoolInfo::new("uwu", "BCnhgdfcgg5aG2a9vAVfnW65Unxt7Qj3YnDyCS4AHfqX"),
        PoolInfo::new("mango", "BKi84JsPEnXT4UBTZtET5h8Uf2y3qLWAnyAZXtMVNK1o"), // verify: from "Mango SOL"
        PoolInfo::new("ded", "BQXDa4DHjHsLNGnE7JhTvhoCuy9Gjzasofxa8QTchYRV"), // verify: from "DEDsol"
        PoolInfo::new("stazzy", "BTnAmxeEKQUKMdKSDNCzhriUAddXny95bjuqSnmrSx6G"),
        PoolInfo::new("pesky_penguins", "BaBrnRoSyVWu4oaSZGQqEUwz6A41xvCtUotpA2UrxvDL"),
        PoolInfo::new("jito_2", "BaQ5BQ2tShQsG6gxSAXZbG6F9jKWMXxASTa8fBgoQab2"), // stale: cached balance >10 epochs old
        PoolInfo::new("daopool", "BbyX1GwUNsfbcoWwnkZDo8sqGmwNDzs2765RpjyQ1pQb"),
        PoolInfo::new("notpet", "BcNh5r1bbLrAkyPZkzLTM44mxPHWE1Ya9f3Jep7enE3P"), // stale: cached balance >10 epochs old
        PoolInfo::new("redelegate", "BoTyh5ptDLbZCbr8yyHkxiwCF3oCkc4477vSc4Wrtujg"),
        PoolInfo::new("luna", "BoxKB3pQd9JxSHtkCfWuaxSjEmb1y9nYfjqpV4iwnwP6"), // verify: from "LunaSOL"
        PoolInfo::new("dynosol", "BqPJdYKKpReEfXHv8kgdmRcBfLToBSHpt1qThtb52GSs"),
        PoolInfo::new("danger_4", "C3WQQ6SdKg9DY4n2F7r9BCjiS8eU8YK8VTcyhytWUPZZ"), // stale: cached balance >10 epochs old
        PoolInfo::new("solsd", "C6xBUyuGtrMEcBuh7WKf9K6cVvtQVyEC3nR3DaSTYpaW"),
        PoolInfo::new("risk_freestacc", "CA7yTjzt8JgkyamhoNveQZP9TMMZFcUyDabsYWBZipt6"),
        PoolInfo::new("windfall", "CQRkqnVvLfscs1g5MUczYRdA3HBMQd3k7qtfFTd6XYc1"),
        PoolInfo::new("watchtower", "CRF1nzFsGDBtg4gsHMUaHCECTz19kJrGsjQQayyFKA7F"),
        PoolInfo::new("xplace", "CTCB72j6RqkzmuzASqfQx6RdhS3RD5nfYicVq5Nd8tDu"),
        PoolInfo::new("danger_5", "CaFsSmBD7iG1nqty9gcBVMGYEEawtW2sENuW3oCKyEVa"), // stale: cached balance >10 epochs old
        PoolInfo::new("alpha", "D1eBo8GuSE3ozAVfwf1iSeZXtpzyjoFK3AcHcGiCHhPZ"), // verify: from "Alpha SOL"
        PoolInfo::new("anz", "D5CRQAGAF2N6jejgjcnzZCg3oEVU6Kjrk1qAKJJsPxaT"), // verify: from "anzSOL"
        PoolInfo::new("tax", "D62EBDqhdqUexqGbhyBvWGWGfqSNT8Cev9JYqW871Ryi"), // verify: from "taxSOL"
        PoolInfo::new("honeycomb", "D8iG47yDtRRnwE3eXyN5nMJoRn55BTKEoMv46eiTrWYA"),
        PoolInfo::new("dual", "D9JhMGji95NwKhfwiLxThjqLsBJ6V98gpEzMz8s78Set"), // verify: from "Dual SOL"
        PoolInfo::new("shiroi", "DB9qQhF3GGWxw4tMBbjiznWLCLa9UxCZUigEtzogw9zc"), // stale: cached balance >10 epochs old
        PoolInfo::new("pengu", "DCRYPZbPkSddAAt3BdU8Sz6oAnwhgREsSzDFJhPh6cbd"),
        PoolInfo::new("maw", "DCnS9jSwtDsXxEnFQC4tQKm8NDbZMkfi4zut4vpWkHRG"),
        PoolInfo::new("adra_lst", "DJ5zc5UhPCAbFhudnw1RqrgcQimUzh5th6WEGtTN12NS"),
        PoolInfo::new("udder_chaos", "DMj2ivDgS8y26H9CojejgkmivgmKrr1QWLDWurLaaizB"),
        PoolInfo::new("radeon", "DQ5sxbJvovkkq83Uu2cq5zS531sAwpDLrQHGhpUT9Y2m"), // stale: cached balance >10 epochs old
        PoolInfo::new("delegate_liquid_staking", "DbTSPoidRbbjpUm5WyecaytURmpNv1WRqhGcjsPdS2o9"),
        PoolInfo::new("steaknet", "Dfrck4LsCWMebzDwjEY7Qg7Q8PGdACtyEctRWvzNj1Rm"),
        PoolInfo::new("axiom", "DoJahc3Eqxoe8kAUWveu1srApZo19tLYLb6pJe11sofg"),
        PoolInfo::new("paragon", "DwjnJy2EzduTJ19dwRGwGFdG9nE6iYaMmdzRWTBYfbcP"),
        PoolInfo::new("legends", "DxvDMbAB5CTWQhVMtCEnKi3RZMJkW31vKkwjWzPRc48y"),
        PoolInfo::new("solanafm", "Dznab3DUrWHPGdCjToTGTgQgaHdxYaRuGgG9LTPTcE7N"),
        PoolInfo::new("digit", "E1dUyGssrE56nMz1pjpbXxgPGVRsKBxchgXZYmSJFJMw"), // verify: from "digitSOL"
        PoolInfo::new("yieldbay", "EB9XQbNCNLesNoo1NScSNv6isbZbPbtzv1dDEuDNyCmg"),
        PoolInfo::new("mexc_2", "ECVFhdhHYpVmHDUD7b6N4Y38HGL2GXHj4wnmqPmjZ4U4"),
        PoolInfo::new("jupiter", "EMjuABxELpYWYEwjkKmQKBNCwdaFAy4QYAs6W9bDQDNw"),
        PoolInfo::new("gotm", "EPqDy4MhG6h5fun6UKfKGzDEFGGHiQh57cwpc8Ld3vJB"),
        PoolInfo::new("mbr", "ER6v9yTBuf338RH9hiPj5FmM4comoJxTPRjJH5cfEAwt"),
        PoolInfo::new("bitget", "EVhp44NGYxxrxhv2NyFyErEKcsiffvssju5K7C5xydye"),
        PoolInfo::new("solana_vibe_station", "EbG4Ti1ruYacPa94mQmicUxPwYGv5b8hAQPUV2pCsUgQ"),
        PoolInfo::new("goosefx", "EdDWp1EzFSLe7pMnFt6pgo6X8PbYApEpreq7bAzcp6Le"),
        PoolInfo::new("paw_2", "Ej13jgxPwoBzcwceyj8371d2kHHgE9Ars6SF692cXmXt"), // verify: from "Paw SOL"; stale: cached balance >10 epochs old
        PoolInfo::new("sanctum", "EjYFnQcNDmfYQqT5B2R2239i781D5wNXrqA2qx2gYJo1"),
        PoolInfo::new("eat_tribe", "EmWViXbym29aSDUEcPkkjv9gkqh6gPsYvgX7pWZDgF1y"),
        PoolInfo::new("trillium_liquid_staked_sol_tri1lst", "EnYhmDgKnHs56R2bwdcepirUVB7n991riGqxe9K4bXUH"),
        PoolInfo::new("tainaker", "EnyvfuWJBNbqXCPuyXMrdP78t9SJtq7rU9niwNcU7ir6"),
        PoolInfo::new("shinobi", "EpH4ZKSeViL5qAHA9QANYVHxdmuzbUH2T79f32DmSCaM"),
        PoolInfo::new("onionlst", "EyTn3xemnJk3wVioTUroPTwh628SgoCyMk6FxhjrmbYR"),
        PoolInfo::new("espres", "EzD116Ef44jnsiJonLN5BPYH8Hmt4paffh7w725MN31y"),
        PoolInfo::new("zippy", "F15nfVkJFAa3H4BaHEb6hQBnmiJZwPYioDiE1yxbc5y4"),
        PoolInfo::new("tinydancer", "F2tMDGNVfHvVBbuAkpXZmPMRrWywE2a8jzH5RQ26zm1r"),
        PoolInfo::new("raiku", "F6LLwairYCMHoAZqRUbCD8HgXmPag1D6YVVKAkS5L4Jb"),
        PoolInfo::new("islanddao", "FFZTyrLxWQc5CprW2zR4g5tEhUXX9fJxE59gRKiSSwVB"),
        PoolInfo::new("juicy", "FKDyJz5tPUy1ArAUba7ziQLbMKzaivRnHiW4FHzCSE9t"),
        PoolInfo::new("bomb", "FMurKhPfmJZGuc8PddLKnbq3sq5SBFih1iiJ1VKVRSYo"), // stale: cached balance >10 epochs old
        PoolInfo::new("layer33", "FQS7JfBjCUiSj6JRHZWqnuM8FNxnDrbaoErCXXe6fAj8"),
        PoolInfo::new("sentinel", "FQVNzbFUGoFeXkjbjNe29FeRWYpuEBwjt8gTuFS1bCf6"),
        PoolInfo::new("kuma", "FZ2F6rHLcaK7GfPpyfDBGofTjfJ3BnA8wMkXbnt21riT"), // verify: from "kumaSOL"
        PoolInfo::new("rockawayx", "FZ9mgo27x9D7rDCSjs4MyY6AKEgAEBZodu2GBA7uYUG6"),
        PoolInfo::new("edgevana", "FZEaZMmrRC3PDPFMzqooKLS2JjoyVkKNd2MkHjr7Xvyq"),
        PoolInfo::new("topvalidators_liquid", "Fe4n5dekXPMmZnp6SWKqr8EZFZxzQ9yfqw6RfNzuRYHZ"),
        PoolInfo::new("portals", "FmwzgFKbh1pE92D1zLRK85RQxnBs3AzBRMSeu87BYLSS"),
        PoolInfo::new("flojo", "Fp2CDREF68AS9temDsY1XgvmLHmWTYKd595CqAdh2Fhr"),
        PoolInfo::new("takisoul_lst", "FyfXwjLQDXeyYqEiuYkTFRHLhWLVmaFsvf5UcggEQe7K"),
        PoolInfo::new("nasdao", "G5j1efg854xgSF1svq9AoSwC7ySbpw4nbbC5ZNMYhctu"),
        PoolInfo::new("refi_hub", "GBHPzZJ1zYyXu2aM28SUDvQv7dXxr4XyVSJbtWHujVFh"),
        PoolInfo::new("staking_facilities", "GYcF4ugBEBBeKMvzDtjBMj2ickUjnxxDiQ5gtjKnU5kT"),
        PoolInfo::new("vault", "GdNXJobf8fbTR5JSE7adxa6niaygjx4EEbnnRaDCHMMW"),
        PoolInfo::new("crypto_com", "GiqwVAud4dH939qajy4F33Cht84kzutxJnGHez4urXnJ"),
        PoolInfo::new("pink", "GmnaSX9RukGHzwSxpPpqxjvLu1Soewia77eSKhvFRDsy"), // verify: from "pinkSOL"; stale: cached balance >10 epochs old
        PoolInfo::new("gumshoe", "Gzr3jLjuGgjbMQi7Uan5jrUzrjWvZd54Lb4ftnhcQu8A"),
        PoolInfo::new("monkedao", "H1hSDP7xx9mtFncRDwgZpoyGB1qbVmqELtcPGNK6Lq94"),
        PoolInfo::new("redelegate_2", "H3eUeig9jkP2pTeDbP51xLchSm7fub3JwETcA5vL7Wzt"),
        PoolInfo::new("solayer", "H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv"),
        PoolInfo::new("bomb_2", "H9hzspKiEEH96skCaVkzqwd6gxzdjwZXnJi2nVi716m3"), // stale: cached balance >10 epochs old
        PoolInfo::new("enhanced_linkage", "H9sni3kem3TCJuFtpi9swVbT8EAigBzP2J79v7TBMr1Y"),
        PoolInfo::new("guardian", "HEyY7sTzfFAPjf3eFsDP4wKx1NCNoGvvPxKBECT4ijgn"),
        PoolInfo::new("zerebro", "HQcb93QUhDbckYWmLYGGudNjdvfjk2p9WEJGsR3PGkSX"),
        PoolInfo::new("point", "HUzBzmwRgi3mfZU4B8tBzCJ5tDxD74wRoTMZ3sYtWjKu"), // verify: from "pointSOL"
        PoolInfo::new("paw", "HacNuw45yT7hcG8mFDLLxakrTJy7N4twN8Xk9Tvuq3SJ"), // verify: from "Paw SOL"
        PoolInfo::new("jpool", "HbJTxftxnXgpePCshA8FubsRj9MW4kfPscfuUfn44fnt"),
        PoolInfo::new("chainflow", "Hck8FfpbhWmsRA3iv48wZnU2istGHj2LuL7M1PHq6kYd"),
        PoolInfo::new("fuse", "HgJtBfBVbhPWnaP1LmAeYjGTgZDdFyy63dC2fgiQfDog"),
        PoolInfo::new("prisma", "HjVeQiauFCfwu23FbNmVkoUnZsLzsfRQMLCr3VVdj8cz"),
        PoolInfo::new("godl", "Hmt8KRhaphNsPZPxrXZY9V9nq3Vz1AQ9MpzfmFQrbJsp"),
        PoolInfo::new("kona", "HnYnzHhZcGzGJ2grxrh4s5rCeeQg2pNCuxLzEBVufEJ2"),
        PoolInfo::new("jag_pool", "Hodkwm8xf43JzRuKNYPGnYJ7V9cXZ7LJGNy96TWQiSGN"),
        PoolInfo::new("solstice", "HqDdLupiWcXV2GEfPSsK76c8yhpayUGrfJWzoNmYFVeM"),
        PoolInfo::new("mas_defi", "HsB5kB85TZ5uyFMR6QeHcVVfF5CPc1bCtbbKp3VG6Sms"),
        PoolInfo::new("hosico", "J5jXr5Ea7kEms4p7ivyJGmUHqGmyc9ec1G8ABH2gBhsd"),
        PoolInfo::new("starpool", "JBV9qdbKkiz1WmszJuL7qGuAYE2suPP6YU5R7gHqCGRe"),
        PoolInfo::new("dic", "JDT3agZFSAGc7ZTrz59gLJtM7VkvJ7T2VvpiHdSX3hMC"), // verify: from "dicSOL"
        PoolInfo::new("overclock", "JECzmc2XbPgS4U8h3ASEw7uytc3p4uFyj2Nw1u4DeBEM"),
        PoolInfo::new("corvus", "UTaViEBKZECiHx4B2knZXZ5fbzLLesj9jFZDkeMzzcD"),
        PoolInfo::new("mallow", "hHt4Yg7FDj2JWQ5ExMMu1Btdahx2w5rmPKsvoXX3PtJ"),
        PoolInfo::new("chive", "t7cpiMhack8XCWkzioHPRLFx2rszY8c6vZBQQxbFehr"),
        PoolInfo::new("bulk", "tjbFnqBF8xwBbU7qg46mU5McehUVJLMU5gVryE25D9F"),
        PoolInfo::new("cult_jare", "vgida1m4TBMgqnjhbZQ6gbSC5Dasgyh6EXtxxDc2LRN"), // stale: cached balance >10 epochs old
        // ---- END GENERATED ----
    ]
});

#[rustfmt::skip]
static POOLS_RETIRED: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    vec![
        // ---- RETIRED ----
        // ---- END RETIRED ----
    ]
});

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

/// Index by pool name for fast lookups
static POOLS_BY_NAME: Lazy<HashMap<String, PoolInfo>> = Lazy::new(|| {
    POOLS_REGISTRY
        .iter()
        .map(|pool| (pool.name.clone(), pool.clone()))
        .collect()
});

/// Index by authority for fast reverse lookups
static POOLS_BY_AUTHORITY: Lazy<HashMap<String, PoolInfo>> = Lazy::new(|| {
    POOLS_REGISTRY
        .iter()
        .map(|pool| (pool.authority.clone(), pool.clone()))
        .collect()
});

/// Get all available pools
#[must_use]
pub fn get_all_pools() -> &'static [PoolInfo] {
    &POOLS_REGISTRY
}

/// Get pool info by name
pub fn get_pool_by_name(name: &str) -> Option<&PoolInfo> {
    POOLS_BY_NAME.get(name)
}

/// Get pool info by authority
pub fn get_pool_by_authority(authority: &str) -> Option<&PoolInfo> {
    POOLS_BY_AUTHORITY.get(authority)
}

/// Get multiple pools by names
#[must_use]
pub fn get_pools_by_names(names: &[&str]) -> Vec<PoolInfo> {
    names
        .iter()
        .filter_map(|name| get_pool_by_name(name))
        .cloned()
        .collect()
}

/// Check if a pool name exists
pub fn pool_exists(name: &str) -> bool {
    POOLS_BY_NAME.contains_key(name)
}

/// Get all pool names
pub fn get_all_pool_names() -> Vec<String> {
    POOLS_REGISTRY
        .iter()
        .map(|pool| pool.name.clone())
        .collect()
}

/// Get all authorities
pub fn get_all_authorities() -> Vec<String> {
    POOLS_REGISTRY
        .iter()
        .map(|pool| pool.authority.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_registry_not_empty() {
        assert!(!POOLS_REGISTRY.is_empty());
        assert!(POOLS_REGISTRY.len() > 30); // 294 today: 33 manual + 261 generated
    }

    #[test]
    fn test_get_pool_by_name() {
        let jito = get_pool_by_name("jito").unwrap();
        assert_eq!(jito.name, "jito");
        assert_eq!(
            jito.authority,
            "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"
        );
    }

    #[test]
    fn test_get_pool_by_authority() {
        let marinade =
            get_pool_by_authority("4bZ6o3eUUNXhKuqjdCnCoPAoLgWiuLYixKaxoa8PpiKk").unwrap();
        assert_eq!(marinade.name, "marinade");
    }

    #[test]
    fn test_unknown_pool() {
        assert!(get_pool_by_name("unknown_pool").is_none());
    }

    #[test]
    fn test_get_multiple_pools() {
        let pools = get_pools_by_names(&["jito", "marinade", "unknown"]);
        assert_eq!(pools.len(), 2);
        assert!(pools.iter().any(|p| p.name == "jito"));
        assert!(pools.iter().any(|p| p.name == "marinade"));
    }

    #[test]
    fn test_pool_exists() {
        assert!(pool_exists("jito"));
        assert!(!pool_exists("nonexistent"));
    }

    #[test]
    fn test_all_pools_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for pool in get_all_pools() {
            assert!(
                names.insert(&pool.name),
                "Duplicate pool name: {}",
                pool.name
            );
        }
    }

    #[test]
    fn test_all_pools_have_valid_authorities() {
        for pool in get_all_pools() {
            // Basic validation - authorities should be base58 strings of length 32-44
            assert!(!pool.authority.is_empty());
            assert!(pool.authority.len() >= 32);
            assert!(pool.authority.len() <= 44);
            // Should only contain base58 characters
            assert!(pool
                .authority
                .chars()
                .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)));
        }
    }

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
}
