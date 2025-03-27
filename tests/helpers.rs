/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct SmugMugOauth1Token {
    token: String,
    secret: String,
}

fn get_smugmug_tokens(path: PathBuf) -> anyhow::Result<SmugMugOauth1Token> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

#[allow(dead_code)]
pub(crate) fn get_full_auth_tokens() -> anyhow::Result<smugmug::v2::Creds> {
    let api_key = std::env::var("SMUGMUG_API_KEY")?;
    let api_secret = std::env::var("SMUGMUG_API_SECRET")?;
    let token_cache = std::env::var("SMUGMUG_AUTH_CACHE")?;
    let tokens = get_smugmug_tokens(token_cache.into())?;

    Ok(smugmug::v2::Creds::from_tokens(
        &api_key,
        Some(&api_secret),
        Some(&tokens.token),
        Some(&tokens.secret),
    ))
}

#[allow(dead_code)]
pub(crate) fn get_read_only_auth_tokens() -> anyhow::Result<smugmug::v2::Creds> {
    let api_key = std::env::var("SMUGMUG_API_KEY")?;

    Ok(smugmug::v2::Creds::from_tokens(
        &api_key,
        None,
        None,
        None,
    ))
}
