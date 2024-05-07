# Dify Client

The Dify Client is a Rust library for interacting with the Dify service. It provides a convenient way to integrate Dify functionality into your Rust applications.

## Installation

To add `dify-client` to your package, add the following to your `Cargo.toml`:

```toml
[dependencies]
dify-client = "0.2"
```

## Test

To run the tests, you need to set the `DIFY_API_KEY` and `DIFY_BASE_URL` environment variables.

```bash
export DIFY_API_KEY=your_api_key
export DIFY_BASE_URL=https://api.dify.io
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
