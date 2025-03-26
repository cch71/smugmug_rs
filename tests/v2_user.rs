/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
mod helpers;

#[cfg(test)]
mod test {
    use crate::helpers;
    use dotenvy::dotenv;
    use smugmug::v2::{Client, User};

    #[tokio::test]
    async fn user_from_id() {
        dotenv().ok();
        let creds = helpers::get_read_only_auth_tokens().unwrap();
        let client = Client::new(creds);
        let user_info = User::user_from_id(client.clone(), "apidemo").await.unwrap();
        println!("User info: {:?}", user_info);
    }

    #[tokio::test]
    async fn authenticated_user_info() {
        dotenv().ok();
        let creds = helpers::get_full_auth_tokens().unwrap();
        let client = Client::new(creds);
        let user_info = User::authenticated_user_info(client.clone()).await.unwrap();
        println!("User info: {:?}", user_info);
    }
}
