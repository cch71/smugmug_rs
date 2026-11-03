/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

//! OAuth1 (RFC 5849) request signing and the SmugMug login steps.
//!
//! Logging in to get an access token/secret takes three steps:
//! 1. [`oauth1_request_token`] gets a temporary request token.
//! 2. The user opens [`oauth1_authorize_url`] in a browser, approves access and is shown a
//!    six digit verifier (PIN) when the callback is `oob`.
//! 3. [`oauth1_access_token`] trades the request token and verifier for the access token,
//!    which is then used with [`crate::v2::Creds::from_tokens`].
//!
//! See [SmugMug OAuth Docs](https://api.smugmug.com/api/v2/doc/tutorial/authorization.html)

use crate::v2::errors::SmugMugError;
use base64::prelude::*;
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use rand::RngExt;
use rand::distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use urlencoding::encode as url_encode;

/// SmugMug endpoint for step 1 of the login
pub const OAUTH1_REQUEST_TOKEN_URL: &str =
    "https://secure.smugmug.com/services/oauth/1.0a/getRequestToken/";

/// SmugMug page the user approves access on in step 2 of the login
pub const OAUTH1_AUTHORIZE_URL: &str = "https://secure.smugmug.com/services/oauth/1.0a/authorize/";

/// SmugMug endpoint for step 3 of the login
pub const OAUTH1_ACCESS_TOKEN_URL: &str =
    "https://secure.smugmug.com/services/oauth/1.0a/getAccessToken/";

/// An OAuth1 token and its secret, either the temporary request token or the access token
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OAuth1Token {
    pub token: String,
    pub secret: String,
}

impl std::fmt::Debug for OAuth1Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth1Token")
            .field("token", &"xxx")
            .field("secret", &"xxx")
            .finish()
    }
}

/// Calculates the OAuth1 HMAC-SHA1 signature (RFC 5849 section 3.4).
///
/// `base_url` is the request url without its query, and `params` holds every `oauth_*`
/// parameter (other than `oauth_signature`) plus the request's query and form parameters,
/// all unencoded.
pub fn oauth1_signature(
    method: &str,
    base_url: &str,
    params: &[(&str, &str)],
    consumer_secret: &str,
    token_secret: Option<&str>,
) -> String {
    let mut encoded_params: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (url_encode(k).into_owned(), url_encode(v).into_owned()))
        .collect();
    encoded_params.sort();
    let param_str = encoded_params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let base_str = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        url_encode(base_url),
        url_encode(&param_str)
    );
    let key = format!(
        "{}&{}",
        url_encode(consumer_secret),
        url_encode(token_secret.unwrap_or(""))
    );

    let mut mac =
        Hmac::<Sha1>::new_from_slice(key.as_bytes()).expect("HMAC can be initialized with key");
    mac.update(base_str.as_bytes());
    BASE64_STANDARD.encode(mac.finalize().into_bytes())
}

/// Creates the OAuth1 `Authorization` header value for a request.
///
/// The url's query parameters are included in the signature.  `token` is the token and its
/// secret, if there is one yet (there isn't for step 1 of the login).
pub fn oauth1_authorization_header(
    method: &str,
    url: &url::Url,
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<(&str, &str)>,
) -> String {
    let nonce: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let timestamp = Utc::now().timestamp().to_string();
    oauth1_authorization_header_with(
        method,
        url,
        consumer_key,
        consumer_secret,
        token,
        &nonce,
        &timestamp,
    )
}

// Creates the Authorization header with the given nonce and timestamp
fn oauth1_authorization_header_with(
    method: &str,
    url: &url::Url,
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<(&str, &str)>,
    nonce: &str,
    timestamp: &str,
) -> String {
    let mut oauth_params = vec![
        ("oauth_consumer_key", consumer_key),
        ("oauth_nonce", nonce),
        ("oauth_signature_method", "HMAC-SHA1"),
        ("oauth_timestamp", timestamp),
        ("oauth_version", "1.0"),
    ];
    if let Some((token, _)) = token {
        oauth_params.push(("oauth_token", token));
    }

    // The signature covers the query too, but the url itself is signed without it
    let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
    let mut all_params = oauth_params.clone();
    all_params.extend(query.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    let base_url = {
        let mut base_url = url.clone();
        base_url.set_query(None);
        base_url.set_fragment(None);
        base_url.to_string()
    };
    let signature = oauth1_signature(
        method,
        &base_url,
        &all_params,
        consumer_secret,
        token.map(|(_, secret)| secret),
    );
    oauth_params.push(("oauth_signature", &signature));

    format!(
        "OAuth {}",
        oauth_params
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, url_encode(v)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Step 1 of the login: gets a temporary request token.
///
/// `callback` is where SmugMug sends the user after they approve access, or `oob` to show
/// them a verifier (PIN) to copy back into the app instead.
pub async fn oauth1_request_token(
    consumer_key: &str,
    consumer_secret: &str,
    callback: &str,
) -> Result<OAuth1Token, SmugMugError> {
    request_oauth1_token(
        reqwest::Method::POST,
        OAUTH1_REQUEST_TOKEN_URL,
        &[("oauth_callback", callback)],
        consumer_key,
        consumer_secret,
        None,
    )
    .await
}

/// Step 2 of the login: the page the user opens to approve access for the request token.
///
/// `access` is `Public` or `Full`, and `permissions` is `Read`, `Add` or `Modify`.
pub fn oauth1_authorize_url(
    request_token: &OAuth1Token,
    access: &str,
    permissions: &str,
) -> Result<url::Url, SmugMugError> {
    Ok(url::Url::parse_with_params(
        OAUTH1_AUTHORIZE_URL,
        &[
            ("oauth_token", request_token.token.as_str()),
            ("access", access),
            ("permissions", permissions),
        ],
    )?)
}

/// Step 3 of the login: trades the request token and the verifier the user got in step 2 for
/// the access token.
pub async fn oauth1_access_token(
    consumer_key: &str,
    consumer_secret: &str,
    request_token: &OAuth1Token,
    verifier: &str,
) -> Result<OAuth1Token, SmugMugError> {
    request_oauth1_token(
        reqwest::Method::GET,
        OAUTH1_ACCESS_TOKEN_URL,
        &[("oauth_verifier", verifier)],
        consumer_key,
        consumer_secret,
        Some((&request_token.token, &request_token.secret)),
    )
    .await
}

// Sends an OAuth1 signed request for a token and returns the oauth_token/oauth_token_secret
// from the form encoded response
async fn request_oauth1_token(
    method: reqwest::Method,
    url: &str,
    query: &[(&str, &str)],
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<(&str, &str)>,
) -> Result<OAuth1Token, SmugMugError> {
    let req_url = url::Url::parse_with_params(url, query)?;
    let auth_header = oauth1_authorization_header(
        method.as_str(),
        &req_url,
        consumer_key,
        consumer_secret,
        token,
    );
    let resp = reqwest::Client::new()
        .request(method, req_url)
        .header("Authorization", auth_header)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    parse_token_response(&resp)
}

// Pulls the token and secret out of a form encoded token response
fn parse_token_response(resp: &str) -> Result<OAuth1Token, SmugMugError> {
    let mut token = None;
    let mut secret = None;
    for (k, v) in url::form_urlencoded::parse(resp.as_bytes()) {
        match k.as_ref() {
            "oauth_token" => token = Some(v.into_owned()),
            "oauth_token_secret" => secret = Some(v.into_owned()),
            _ => {}
        }
    }
    match (token, secret) {
        (Some(token), Some(secret)) => Ok(OAuth1Token { token, secret }),
        _ => Err(SmugMugError::Auth(format!(
            "OAuth token missing from response: {}",
            resp
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example from https://developer.x.com/en/docs/authentication/oauth-1-0a/creating-a-signature
    const CONSUMER_KEY: &str = "xvz1evFS4wEEPTGEFPHBog";
    const CONSUMER_SECRET: &str = "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw";
    const TOKEN: &str = "370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb";
    const TOKEN_SECRET: &str = "LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE";
    const NONCE: &str = "kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg";
    const TIMESTAMP: &str = "1318622958";
    const EXPECTED_SIGNATURE: &str = "hCtSmYh+iHYCEqBWrE7C7hYmtUk=";

    #[test]
    fn oauth1_signature_matches_published_example() {
        let params = [
            (
                "status",
                "Hello Ladies + Gentlemen, a signed OAuth request!",
            ),
            ("include_entities", "true"),
            ("oauth_consumer_key", CONSUMER_KEY),
            ("oauth_nonce", NONCE),
            ("oauth_signature_method", "HMAC-SHA1"),
            ("oauth_timestamp", TIMESTAMP),
            ("oauth_token", TOKEN),
            ("oauth_version", "1.0"),
        ];
        let signature = oauth1_signature(
            "POST",
            "https://api.twitter.com/1.1/statuses/update.json",
            &params,
            CONSUMER_SECRET,
            Some(TOKEN_SECRET),
        );
        assert_eq!(signature, EXPECTED_SIGNATURE);
    }

    // Same example with the parameters in the query, as the client sends them
    #[test]
    fn oauth1_header_signs_query_params() {
        let url = url::Url::parse_with_params(
            "https://api.twitter.com/1.1/statuses/update.json",
            &[
                ("include_entities", "true"),
                (
                    "status",
                    "Hello Ladies + Gentlemen, a signed OAuth request!",
                ),
            ],
        )
        .unwrap();
        let header = oauth1_authorization_header_with(
            "POST",
            &url,
            CONSUMER_KEY,
            CONSUMER_SECRET,
            Some((TOKEN, TOKEN_SECRET)),
            NONCE,
            TIMESTAMP,
        );
        assert!(header.starts_with("OAuth oauth_consumer_key=\"xvz1evFS4wEEPTGEFPHBog\""));
        assert!(header.contains(&format!(
            "oauth_signature=\"{}\"",
            url_encode(EXPECTED_SIGNATURE)
        )));
        assert!(header.contains(&format!("oauth_token=\"{}\"", TOKEN)));
        // Query params are signed but not sent in the header
        assert!(!header.contains("status"));
    }

    #[test]
    fn oauth1_header_without_token() {
        let url = url::Url::parse_with_params(OAUTH1_REQUEST_TOKEN_URL, &[("oauth_callback", "oob")])
            .unwrap();
        let header = oauth1_authorization_header("POST", &url, CONSUMER_KEY, CONSUMER_SECRET, None);
        assert!(!header.contains("oauth_token="));
        assert!(header.contains("oauth_signature=\""));
    }

    #[test]
    fn parses_token_response() {
        let token =
            parse_token_response("oauth_token=abc&oauth_token_secret=d%2Bef&oauth_callback_confirmed=true")
                .unwrap();
        assert_eq!(token.token, "abc");
        assert_eq!(token.secret, "d+ef");
        assert!(parse_token_response("oauth_problem=bad").is_err());
    }

    #[test]
    fn authorize_url_has_token_and_access() {
        let token = OAuth1Token {
            token: "abc".to_string(),
            secret: "def".to_string(),
        };
        let url = oauth1_authorize_url(&token, "Full", "Modify").unwrap();
        assert_eq!(
            url.as_str(),
            "https://secure.smugmug.com/services/oauth/1.0a/authorize/?oauth_token=abc&access=Full&permissions=Modify"
        );
    }
}
