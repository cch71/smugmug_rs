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
                v.$r.client = Some($c.clone());
                v.$r
            })
    }};
}

macro_rules! obj_update_from_url {
    ( $c:expr, $url: expr, $d: expr,$rt: ty, $r: ident) => {{
        let params = vec![("_verbosity", "1")];
        $c.patch::<$rt>($url, $d, Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|mut v| {
                v.$r.client = Some($c.clone());
                v.$r
            })
    }};
}

macro_rules! obj_update_from_uri {
    ( $c:expr, $uri: expr, $d: expr,$rt: ty, $r: ident) => {{
        let req_url = url::Url::parse(API_ORIGIN)?.join($uri)?;
        obj_update_from_url!($c, req_url.as_str(), $d, $rt, $r)
    }};
}

macro_rules! objs_from_id_slice {
    ( $c:expr, $ids:expr, $uri:expr, $rt: ty, $r: ident) => {{
        if $ids.is_empty() {
            return Ok(Vec::new());
        }
        let params = vec![("_verbosity", "1")];
        let req_url = url::Url::parse(API_ORIGIN)?
            .join($uri)?
            .join($ids.join(",").as_str())?;
        // println!("multi-get url: {}", req_url.as_str());
        $c.get::<$rt>(req_url.as_str(), Some(&params))
            .await?
            .payload
            .ok_or(SmugMugError::ResponseMissing())
            .map(|v| {
                v.$r.into_iter()
                    .map(|mut v| {
                        v.client = Some($c.clone());
                        v
                    })
                    .collect()
            })
    }};
}

// Streams the items of a paged listing.  The given params (plus `_verbosity`) are applied to
// every page, since SmugMug's `Pages.NextPage` uri drops most of them.  The optional last
// argument maps each item; by default the client is set on it.
macro_rules! stream_children_from_url {
    ( $c:expr, $url: expr, $params:expr, $rt: ty, $r: ident) => {{
        stream_children_from_url!($c, $url, $params, $rt, $r, |mut item| {
            item.client = Some($c.clone());
            item
        })
    }};
    ( $c:expr, $url: expr, $params:expr, $rt: ty, $r: ident, |$item:pat_param| $map: expr) => {{
        try_stream! {
            if let Some(url) = $url {
                let mut params: Vec<(&str, &str)> = ($params).to_vec();
                params.push(("_verbosity", "1"));
                let mut req_url = crate::v2::client::api_url_with_params(&url, &params)?;
                loop {
                    let resp = $c.get::<$rt>(req_url.as_str(), None)
                        .await?
                        .payload
                        .ok_or(SmugMugError::ResponseMissing())?;
                    for $item in resp.$r {
                        yield $map
                    }

                    if let Some(next_page) = resp.pages.and_then(|p| p.next_page) {
                        req_url = crate::v2::client::api_url_with_params(&next_page, &params)?;
                    } else {
                        break;
                    }
                }
            }
        }
    }};
}

pub(crate) use {
    obj_from_url, obj_update_from_uri, obj_update_from_url, objs_from_id_slice,
    stream_children_from_url,
};
