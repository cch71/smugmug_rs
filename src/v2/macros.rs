/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */

macro_rules! obj_from_url {
    ( $c:expr, $url: expr, $rt: ty, $r: ident) => {{
        let params = vec![("_verbosity", "1")];
        $c.get::<$rt>($url, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.$r.client = $c.clone();
                v.$r
            })
    }};
}

macro_rules! objs_from_id_slice {
    ( $c:expr, $ids:expr, $uri:expr, $rt: ty, $r: ident) => {{
        let params = vec![("_verbosity", "1")];
        let req_url = url::Url::parse(API_ORIGIN)?
            .join($uri)?
            .join($ids.join(",").as_str())?;
        $c.get::<$rt>(req_url.as_str(), Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|v| {
                v.$r.into_iter()
                    .map(|mut v| {
                        v.client = $c.clone();
                        v
                    })
                    .collect()
            })
    }};
}

pub(crate) use {obj_from_url, objs_from_id_slice};
