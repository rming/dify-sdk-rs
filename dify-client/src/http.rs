//! This module re-exports some common items most developers need from the [reqwest](https://docs.rs/reqwest) crate.
//!
//! The re-exported items include:
//! - `header`: Types and functions for working with HTTP headers.
//! - `multipart`: Types and functions for working with multipart requests.
//! - `Method`: An enum representing HTTP methods.
//! - `Request`: A builder for making HTTP requests.
//! - `Response`: A response to an HTTP request.
//!
//! This module is intended to provide a convenient way to access commonly used items from the reqwest crate.
pub use reqwest::{header, multipart, Method, Request, Response};
