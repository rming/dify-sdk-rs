# Dify Client

The Dify Client is a Rust library for interacting with the Dify service. It provides a convenient way to integrate Dify functionality into your Rust applications.

## Installation

To add `dify-client` to your package, add the following to your `Cargo.toml`:

```toml
[dependencies]
dify-client = "0.3"
```

By default, the library uses the `default-tls` feature, which uses the `reqwest` crate with the default TLS backend. If you want to use the `rustls` TLS backend, you can enable the `rustls-tls` feature (which avoids depending on native libraries like `openssl`):

```toml
[dependencies]
dify-client = { version = "0.3", default-features = false, features = ["rustls-tls"] }
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
cargo doc --no-deps --lib --open
```
