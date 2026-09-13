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
