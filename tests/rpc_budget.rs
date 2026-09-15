//! End-to-end tests against a local fake RPC, focused on RPC *cost*.
//!
//! The registry grew 61 -> 294 pools, and `fetch_all_pools` issues one
//! `getProgramAccounts` per pool. On a rate-limited endpoint the thing that
//! hurts is request COUNT, so these tests pin it: N pools must cost exactly N
//! requests, and a failing pool must not retry more than the configured budget.
//!
//! No mock-HTTP dependency — a plain `TcpListener` on a thread is enough and
//! keeps the dev-dependency list unchanged.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use solana_pools_data_lib::PoolsDataClient;

/// A fake JSON-RPC endpoint that counts requests and replays a canned body.
struct FakeRpc {
    url: String,
    hits: Arc<AtomicUsize>,
}

/// `body_for` receives the request body and returns the JSON-RPC response body.
fn spawn_fake_rpc(body_for: impl Fn(&str) -> String + Send + 'static) -> FakeRpc {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_thread = Arc::clone(&hits);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut raw = Vec::new();
            let mut buf = [0u8; 4096];

            // Read headers, then exactly Content-Length bytes of body.
            let (head_end, content_len) = loop {
                match stream.read(&mut buf) {
                    Ok(0) => break (None, 0),
                    Ok(n) => {
                        raw.extend_from_slice(&buf[..n]);
                        if let Some(p) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                            let head = String::from_utf8_lossy(&raw[..p]).to_lowercase();
                            let len = head
                                .split("content-length:")
                                .nth(1)
                                .and_then(|s| s.split('\r').next())
                                .and_then(|s| s.trim().parse::<usize>().ok())
                                .unwrap_or(0);
                            break (Some(p + 4), len);
                        }
                    }
                    Err(_) => break (None, 0),
                }
            };
            let Some(head_end) = head_end else { continue };
            while raw.len() < head_end + content_len {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => raw.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }

            hits_thread.fetch_add(1, Ordering::SeqCst);
            let req_body = String::from_utf8_lossy(&raw[head_end..]).to_string();
            let body = body_for(&req_body);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });

    FakeRpc {
        url: format!("http://127.0.0.1:{port}"),
        hits,
    }
}

/// Echo the request id so the client's id-matching validation passes.
fn id_of(req: &str) -> u64 {
    req.split("\"id\":")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).find(|t| !t.is_empty()))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}

fn empty_result(req: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":{},"result":[]}}"#, id_of(req))
}

#[tokio::test]
async fn n_pools_cost_exactly_n_requests() {
    // The load guarantee that matters on a rate-limited endpoint: no hidden
    // fan-out, no per-pool extra round trip. If this ever regresses, a full
    // 294-pool refresh silently multiplies.
    let fake = spawn_fake_rpc(empty_result);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let names = ["jito", "marinade", "blazestake", "jupiter", "helius"];
    let out = client.fetch_pools(&names).await.expect("fetch");

    assert_eq!(out.len(), names.len(), "every pool should resolve");
    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        names.len(),
        "expected exactly one RPC request per pool"
    );
}

#[tokio::test]
async fn reserve_only_pool_succeeds_end_to_end_with_empty_data() {
    // Task 7's behaviour through the REAL fetch path, not just the statistics
    // helper: an empty getProgramAccounts result is a success carrying zeroed
    // data, because ~81 discovered pools legitimately hold everything in
    // reserve. Previously this returned Err(NoStakeAccounts).
    let fake = spawn_fake_rpc(empty_result);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let out = client.fetch_pools(&["jito"]).await.expect("empty must not be an error");
    let pool = out.get("jito").expect("pool present");

    assert!(pool.stake_accounts.is_empty());
    assert_eq!(pool.statistics.total_accounts, 0);
    assert_eq!(pool.statistics.total_lamports, 0);
    assert_eq!(pool.statistics.validator_count, 0);
    assert!(pool.validator_distribution.is_empty());
    assert_eq!(pool.authority, "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS");
}

#[tokio::test]
async fn retries_are_bounded_by_the_configured_budget() {
    // The amplification hazard: a struggling endpoint returns errors, the
    // client retries, and load rises exactly when the endpoint can least take
    // it. Worst case must stay pools * (1 + retry_attempts), never unbounded.
    let fake = spawn_fake_rpc(|req| {
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32005,"message":"Node is behind"}}}}"#,
            id_of(req)
        )
    });
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(2)
        .retry_base_delay(1)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let _ = client.fetch_pools(&["jito"]).await;

    let hits = fake.hits.load(Ordering::SeqCst);
    assert!(
        hits <= 3,
        "1 attempt + 2 retries = at most 3 requests, got {hits}"
    );
    assert!(hits >= 2, "retries should actually happen, got {hits}");
}

#[tokio::test]
async fn unknown_pool_names_cost_zero_requests() {
    // Guards against a typo'd or retired name silently burning RPC budget.
    let fake = spawn_fake_rpc(empty_result);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let err = client.fetch_pools(&["definitely_not_a_pool"]).await;
    assert!(err.is_err(), "unknown pool should error, not silently pass");
    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        0,
        "an unknown name must not reach the network"
    );
}

#[tokio::test]
async fn retries_consume_rate_limit_permits() {
    // The limiter used to be awaited once BEFORE the retry loop, so only the
    // first attempt was paced and every retry bypassed the configured rate —
    // hitting a struggling endpoint faster exactly when it is already failing.
    // With the limiter inside the retry closure, 3 attempts at 1 req/s must take
    // roughly 2 seconds (first is free from the burst allowance, then 1/s).
    let fake = spawn_fake_rpc(|req| {
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32005,"message":"rate limited"}}}}"#,
            id_of(req)
        )
    });
    let client = PoolsDataClient::builder()
        .rate_limit(1)
        .retry_attempts(2)
        .retry_base_delay(1) // backoff ~0, so elapsed time is the limiter's doing
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let started = std::time::Instant::now();
    let _ = client.fetch_pools(&["jito"]).await;
    let elapsed = started.elapsed();

    assert_eq!(fake.hits.load(Ordering::SeqCst), 3, "1 attempt + 2 retries");
    assert!(
        elapsed >= std::time::Duration::from_millis(1500),
        "retries bypassed the rate limiter: 3 attempts at 1/s took only {elapsed:?}"
    );
}

/// Fails every request naming `fail_for`; empty success for everything else.
fn spawn_partial_failure_rpc(fail_for: &'static str) -> FakeRpc {
    spawn_fake_rpc(move |req| {
        if req.contains(fail_for) {
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32005,"message":"rate limited"}}}}"#,
                id_of(req)
            )
        } else {
            empty_result(req)
        }
    })
}

#[tokio::test]
async fn fetch_pools_hides_partial_failure_but_strict_does_not() {
    // The hazard: fetch_pools returns Ok when ANY pool succeeds and drops the
    // rest. On a 429-ing endpoint a 294-pool refresh can return one row and
    // still look successful. A writer treating "absent" as "delete" would then
    // wipe most of a table.
    const JITO_AUTH: &str = "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS";
    let names = ["jito", "marinade", "blazestake"];

    // Lenient: silently returns only the survivors.
    let fake = spawn_partial_failure_rpc(JITO_AUTH);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");
    let lenient = client.fetch_pools(&names).await.expect("lenient returns Ok");
    assert_eq!(lenient.len(), 2, "one pool silently vanished");
    assert!(!lenient.contains_key("jito"));

    // Strict: same conditions, refuses the partial result.
    let fake2 = spawn_partial_failure_rpc(JITO_AUTH);
    let client2 = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake2.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");
    let err = client2
        .fetch_pools_strict(&names)
        .await
        .expect_err("strict must reject a partial result");

    match err {
        solana_pools_data_lib::PoolsDataError::BatchOperationFailed { successful, failed } => {
            assert_eq!(successful, 2);
            assert_eq!(failed, 1);
        }
        other => panic!("expected BatchOperationFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn strict_returns_all_pools_when_none_fail() {
    // Strict must not be pessimistic: a fully successful refresh returns
    // everything, including reserve-only pools with zero stake accounts.
    let fake = spawn_fake_rpc(empty_result);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let names = ["jito", "marinade", "blazestake"];
    let out = client.fetch_pools_strict(&names).await.expect("all succeeded");
    assert_eq!(out.len(), names.len());
    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        names.len(),
        "strict must not cost extra requests"
    );
}

#[tokio::test]
async fn strict_rejects_a_request_naming_a_pool_that_does_not_exist() {
    // get_pools_by_names filter_maps unknown names away before any task is
    // created, so strict never learns the name was requested. A typo'd or
    // retired-and-removed name would therefore return Ok with fewer rows than
    // asked for — exactly the silent shortfall strict exists to prevent.
    let fake = spawn_fake_rpc(empty_result);
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let err = client
        .fetch_pools_strict(&["jito", "jtoi"])
        .await
        .expect_err("an unknown name must fail the whole request");

    let msg = err.to_string();
    assert!(msg.contains("jtoi"), "error should name the bad pool, got: {msg}");
    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        0,
        "an unresolvable request must not spend RPC budget at all"
    );
}

#[tokio::test]
async fn a_redirecting_endpoint_cannot_multiply_requests() {
    // reqwest follows up to 10 redirects by default, which multiplies requests
    // BELOW the layer the other tests count: they saw "1 request per pool" while
    // the wire saw 11 (measured). A 294-pool refresh would become 3,234 requests
    // before retries compound it. Realistic triggers are mundane — a provider
    // domain move, an http->https upgrade, a trailing-slash redirect.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let h = Arc::clone(&hits);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            let mut buf = [0u8; 8192];
            let _ = s.read(&mut buf);
            h.fetch_add(1, Ordering::SeqCst);
            let resp = format!(
                "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://127.0.0.1:{port}/next\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = s.write_all(resp.as_bytes());
            let _ = s.flush();
        }
    });

    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(0)
        .build(&format!("http://127.0.0.1:{port}"))
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let result = client.fetch_pools(&["jito"]).await;

    assert!(
        result.is_err(),
        "a redirect must surface as an error, not be followed"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "redirects were followed and multiplied the per-pool request cost"
    );
}

/// One real jsonParsed stake account, with the lockup timestamp injectable.
fn stake_account_json(unix_timestamp: &str) -> String {
    format!(
        r#"{{"pubkey":"1YayC3Pwb46y1DJTeV3PSzAv23cT9k48RS2netCfJqz","account":{{"lamports":31976057508114,"data":{{"program":"stake","parsed":{{"info":{{"meta":{{"authorized":{{"staker":"6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS","withdrawer":"6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"}},"lockup":{{"custodian":"11111111111111111111111111111111","epoch":0,"unixTimestamp":{unix_timestamp}}},"rentExemptReserve":"2282880"}},"stake":{{"creditsObserved":1213524427,"delegation":{{"activationEpoch":"884","deactivationEpoch":"18446744073709551615","stake":"31976055841874","voter":"7tKWFaaLi2FJSqukHxUrnXph8M3ynrqn3kEkKPpgcNHZ"}}}}}},"type":"delegated"}},"space":200}},"owner":"Stake11111111111111111111111111111111111111","executable":false,"rentEpoch":18446744073709551615,"space":200}}}}"#
    )
}

#[tokio::test]
async fn a_negative_lockup_timestamp_does_not_break_the_pool() {
    // Solana's UnixTimestamp is i64 and the runtime does not require it to be
    // positive. Declaring it u64 meant one account with a negative lockup failed
    // to deserialize — and because the whole account array decodes at once, the
    // ENTIRE pool returned ParseError.
    //
    // That is a cheap denial of service: anyone can create a 200-byte stake
    // account naming a victim pool's authority as staker (keeping themselves as
    // withdrawer, so the pool cannot touch it) with a negative lockup. It matches
    // the offset-12 filter, and every fetch of that pool fails from then on.
    for ts in ["0", "-1", "-62135596800"] {
        let body = stake_account_json(ts);
        let fake = spawn_fake_rpc(move |req| {
            format!(r#"{{"jsonrpc":"2.0","id":{},"result":[{body}]}}"#, id_of(req))
        });
        let client = PoolsDataClient::builder()
            .rate_limit(1000)
            .retry_attempts(0)
            .build(&fake.url)
            .and_then(PoolsDataClient::from_config)
            .expect("client");

        let out = client
            .fetch_pools(&["jito"])
            .await
            .unwrap_or_else(|e| panic!("lockup unixTimestamp {ts} broke the pool: {e}"));
        let pool = out.get("jito").expect("pool present");
        assert_eq!(pool.stake_accounts.len(), 1, "account dropped for ts {ts}");
        assert_eq!(
            pool.stake_accounts[0].lockup.unix_timestamp,
            ts.parse::<i64>().unwrap(),
            "lockup timestamp must round-trip with its sign"
        );
    }
}

#[tokio::test]
async fn a_permanent_error_is_not_retried() {
    // The error type has always classified what is worth retrying, but the
    // verdict was only read AFTER the loop spent its whole budget. A
    // deterministic failure (here invalid-params, -32602) therefore cost
    // 1+retry_attempts requests per pool — across 294 pools that turns one
    // permanent mistake into hundreds of pointless requests against an endpoint
    // that is usually already rate-limiting.
    let fake = spawn_fake_rpc(|req| {
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"invalid params"}}}}"#,
            id_of(req)
        )
    });
    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(5)
        .retry_base_delay(1)
        .build(&fake.url)
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let _ = client.fetch_pools(&["jito"]).await;

    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        1,
        "a non-retryable error must cost exactly one request despite retry_attempts(5)"
    );
}

#[tokio::test]
async fn a_redirect_is_not_retried_either() {
    // A 3xx means the configured URL is not canonical. Retrying cannot fix that,
    // so it is classified as configuration rather than network. Without this the
    // no-redirect policy traded 11 followed hops for 6 retried failures.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let h = Arc::clone(&hits);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            let mut buf = [0u8; 8192];
            let _ = s.read(&mut buf);
            h.fetch_add(1, Ordering::SeqCst);
            let resp = "HTTP/1.1 308 Permanent Redirect\r\nLocation: https://canonical.example/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = s.write_all(resp.as_bytes());
            let _ = s.flush();
        }
    });

    let client = PoolsDataClient::builder()
        .rate_limit(1000)
        .retry_attempts(5)
        .retry_base_delay(1)
        .build(&format!("http://127.0.0.1:{port}"))
        .and_then(PoolsDataClient::from_config)
        .expect("client");

    let _ = client.fetch_pools(&["jito"]).await;

    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "a redirect must not be retried; it is a configuration error, not a transient one"
    );
}
