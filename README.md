# Dify SDK for Rust

## Description

This is the Rust SDK for Dify. It allows you to interact with the Dify API in a more convenient and type-safe way.

## Installation

To add `dify-client` to your package, add the following to your `Cargo.toml`:

```toml
[dependencies]
dify-client = "0.1.0"
```

## Test

To run the tests, you need to set the `DIFY_API_KEY` and `DIFY_BASE_URL` environment variables. You can do this by creating a `.env` file in the root of the project and adding the following:

```env
DIFY_API_KEY=your_api_key
DIFY_BASE_URL=https://api.dify.io
```

Then you can run the tests with:

```sh
cargo test
# cargo test -- --nocapture
# cargo test test_feedback_message -- --nocapture
```

## Docs

To generate the documentation, run:

```sh
# cargo clean --doc
cargo doc --no-deps --lib  --document-private-items --open
```
