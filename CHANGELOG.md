# Changelog

All notable changes to this crate are recorded here. Add entries under `[Unreleased]` as you
make changes. `cargo release` turns that section into the version's section, and the release
workflow publishes it as the GitHub release notes for the version's tag.

## [Unreleased]

## [0.10.0] - 2026-11-03

## [0.9.0] - 2026-11-03

- Fixed the build with `rand` 0.10 and `hmac` 0.13.
- **Behaviour change:** paged listings now send their query parameters on every page.
  Before, they were dropped from all requests, so `Node::children` type and sort filters had
  no effect. Callers that passed a narrow filter but relied on getting every child type back
  now get only the filtered type.
- Empty listings no longer fail to parse.
- `Image` has `date_time_original`, `serial`, `original_width`, `original_height` and
  `original_size`. They're optional, so JSON saved by older versions still loads.
- Added `Image::metadata`, `ImageMetadata` and `Album::images_with_metadata`.
- Added `Image::largest_video`, `LargestVideo` and `LargestVideo::download`.
- Added `Image::key_with_serial`.
- Added `User::deleted_albums` and `User::deleted_folders`.
- Added the `v2::oauth` module with public OAuth1 signing and login helpers.
- `Image::get_archive` returns `ClientNotFound` instead of panicking when the image has no
  client.
- `Node::create_album` returns the new `SmugMugError::NotAFolder` instead of panicking when
  the node can't have children.
- Response codes that aren't in `ApiErrorCodes` are returned as `ApiResponse` with SmugMug's
  message, instead of `ApiResponseCode`.
- `ApiErrorCodes::Conflict` is 409. It was 407.
- `Creds` debug output reports whether the token secret is set, rather than checking the
  access token.
- The `wasm32-unknown-unknown` target builds again. It enables getrandom's `wasm_js` backend.
