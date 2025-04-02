/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use dotenvy::dotenv;
use serde::Deserialize;
use smugmug::v2::Client;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Once, OnceLock};

#[allow(dead_code)]
static LOGGER_INIT: Once = Once::new();

#[allow(dead_code)]
static FULL_CREDS_CLIENT: OnceLock<Client> = OnceLock::new();

#[allow(dead_code)]
static READ_ONLY_CREDS_CLIENT: OnceLock<Client> = OnceLock::new();

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
#[allow(dead_code)]
fn init_logger_and_env() {
    dotenv().ok();
    env_logger::init();
}
#[allow(dead_code)]
pub(crate) fn get_full_client() -> Client {
    LOGGER_INIT.call_once(init_logger_and_env);
    FULL_CREDS_CLIENT.get_or_init(|| {
        let creds = get_full_auth_tokens().unwrap();
        Client::new(creds)
    }).clone()
}
#[allow(dead_code)]
pub(crate) fn get_read_only_client() -> Client {
    LOGGER_INIT.call_once(init_logger_and_env);
    READ_ONLY_CREDS_CLIENT.get_or_init(|| {
        let creds = get_read_only_auth_tokens().unwrap();
        Client::new(creds)
    }).clone()
}
