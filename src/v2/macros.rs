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

macro_rules! stream_children_from_url {
    ( $c:expr, $url: expr, $params:expr, $rt: ty, $r: ident) => {{
        let params = vec![("_verbosity", "1")];

        try_stream! {
            if let Some(url) = $url {
                // The Pages->NextPage doesn't include verbosity so parsing original params
                // and dealing with verbosity seperately
                let mut req_url = url::Url::parse_with_params(API_ORIGIN, $params)?.join(url)?;
                loop {
                    let resp = $c.get::<$rt>(
                        req_url.as_str(), Some(&params)
                    ).await?
                    .payload
                    .ok_or(SmugMugError::ResponseMissing())?;
                    for mut item in resp.$r {
                        item.client = Some($c.clone());
                        yield item
                    }

                    if let Some(next_page) = resp.pages.and_then(|p| p.next_page) {
                        req_url = url::Url::parse(API_ORIGIN)?.join(&next_page)?;
                    } else {
                        break;
                    }
                }
            }
        }
    }};
}

pub(crate) use {obj_from_url, objs_from_id_slice, stream_children_from_url};
