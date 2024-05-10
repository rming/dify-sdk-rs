//! Dify client library.
//!
//! # Examples
//!
//! ## Client with single api key
//!
//! ```no_run
//! use dify_client::{request, Config, Client};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config {
//!         base_url: "https://api.dify.ai".into(),
//!         api_key: "API_KEY".into(),
//!         timeout: Duration::from_secs(60),
//!     };
//!     let client = Client::new_with_config(config);
//!
//!     // Use the client
//!     let data = request::ChatMessagesRequest {
//!         query: "What are the specs of the iPhone 13 Pro Max?".into(),
//!         user: "afa".into(),
//!         ..Default::default()
//!     };
//!     let result = client.api().chat_messages(data).await;
//!     println!("{:?}", result);
//! }
//! ```
//!
//! ## Client with multiple api keys
//!
//! ```no_run
//! use dify_client::{http::header, request, Config, Client};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config {
//!         base_url: "https://api.dify.ai".into(),
//!         api_key: "API_KEY_DEFAULT".into(),
//!         timeout: Duration::from_secs(100),
//!     };
//!     // The client can be safely shared across multiple threads
//!     let client = Client::new_with_config(config);
//!     
//!     // Use the client
//!     let data = request::ChatMessagesRequest {
//!         query: "What are the specs of the iPhone 13 Pro Max?".into(),
//!         user: "afa".into(),
//!         ..Default::default()
//!     };
//!     // Reuse the client with a new api key
//!     let mut api = client.api();
//!     let result = api.chat_messages(data.clone()).await;
//!     println!("{:?}", result);
//!     // Override the api key
//!     api.before_send(|mut req| {
//!         // rewrite the authorization header
//!         let mut auth = header::HeaderValue::from_static("Bearer API_KEY_OVERRIDE");
//!         auth.set_sensitive(true);
//!         req.headers_mut().insert(header::AUTHORIZATION, auth);
//!         req
//!     });
//!     let result = api.chat_messages(data).await;
//!     println!("{:?}", result);
//! }
//! ```
//! For more API methods, refer to the [`Api`](api/struct.Api.html) struct.

pub mod api;
pub mod client;
pub mod http;
pub mod request;
pub mod response;

pub use client::*;
