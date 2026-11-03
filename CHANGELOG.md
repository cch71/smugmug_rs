# Changelog

All notable changes to this crate are recorded here. Add entries under `[Unreleased]` as you
make changes. `cargo release` turns that section into the version's section, and the release
workflow publishes it as the GitHub release notes for the version's tag.

## [Unreleased]

- Fixed the build with `rand` 0.10 and `hmac` 0.13.
- Paged listings now send their query parameters on every page. Before, they were dropped
  from all requests, so `Node::children` type and sort filters had no effect.
- Empty listings no longer fail to parse.
- `Image` has `date_time_original`, `serial`, `original_width`, `original_height` and
  `original_size`. They're optional, so JSON saved by older versions still loads.
- Added `Image::metadata`, `ImageMetadata` and `Album::images_with_metadata`.
- Added `Image::largest_video`, `LargestVideo` and `LargestVideo::download`.
- Added `Image::key_with_serial`.
- Added `User::deleted_albums` and `User::deleted_folders`.
- Added the `v2::oauth` module with public OAuth1 signing and login helpers.
