# AGENTS.md

Guidance for coding agents working on `smugmug_rs`, the Rust client library for the SmugMug
APIv2, published as the `smugmug` crate. See [README.md](README.md) for usage, and the
[SmugMug API Docs](https://api.smugmug.com/api/v2/doc) for the endpoints.

## Rules

### Reuse shared code; don't reimplement it

Ensure that duplicate functionality is combined into common functions rather than
reimplemented.

- Before writing a helper, search `src/` for one that already does the job, and use it. If it
  needs to do more, extend it. Existing shared code includes:
  - **Fetching, updating and listing objects:** the macros in `src/v2/macros.rs`
    (`obj_from_url!`, `obj_update_from_url!`, `objs_from_id_slice!` and
    `stream_children_from_url!`).
  - **Building API urls and re-applying query params when paging:** `api_url_with_params` in
    `src/v2/client.rs`.
  - **Lenient parsing:** the parsers in `src/v2/parsers.rs` for fields SmugMug returns as either
    strings or numbers.
  - **OAuth1 signing:** `src/v2/oauth.rs`.
  - **HTTP requests:** the `Client` get, post, patch and binary download methods.
- If you find yourself copying a block of code, move it into one shared function and call that
  from both places. Do this in the same change.

### Record changes in CHANGELOG.md

Note user-visible changes under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md), not in the
README. `cargo release` turns that section into the version's section, and the release
workflow publishes it as the GitHub release notes. Don't add version headings by hand, and
don't run `cargo release --execute`.

## Checks before finishing

- `cargo build`, `cargo clippy --all-targets` and `cargo test` add no new errors or warnings.
- Run live tests marked `#[ignore]` only by name. Don't use `--include-ignored`: some of those
  tests create or change albums in a real SmugMug account.
