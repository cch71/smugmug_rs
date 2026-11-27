/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

//! Checks that run on the browser (WASM) target, where the standard library has no clock or
//! random number source.  Run with `wasm-pack test --node -- --test wasm`.
#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use smugmug::jiff::Timestamp;
use smugmug::v2::oauth::oauth1_authorization_header;
use wasm_bindgen_test::wasm_bindgen_test;

// The current time comes from JavaScript, since `std` has no clock on this target
#[wasm_bindgen_test]
fn current_time_is_available() {
    assert!(Timestamp::now() > Timestamp::UNIX_EPOCH);
}

// Every API request is signed with a timestamp and a random nonce
#[wasm_bindgen_test]
fn signs_requests() {
    let url = url::Url::parse("https://api.smugmug.com/api/v2!authuser").unwrap();
    let header = oauth1_authorization_header("GET", &url, "key", "secret", Some(("tok", "sec")));
    let timestamp: i64 = header
        .split("oauth_timestamp=\"")
        .nth(1)
        .and_then(|v| v.split('"').next())
        .and_then(|v| v.parse().ok())
        .expect("oauth_timestamp in the header");
    // Within a day of the clock
    assert!((Timestamp::now().as_second() - timestamp).abs() < 86_400);
}
