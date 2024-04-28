//! Dify client library.
//!
//! # Examples
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
//!     let msg = request::ChatMessageRequest {
//!         query: "What are the specs of the iPhone 13 Pro Max?".into(),
//!         user: "afa".into(),
//!         ..Default::default()
//!     };
//!     let result = client.chat_messages(msg).await;
//!     println!("{:?}", result);
//! }
//! ```

pub mod client;
pub mod request;
pub mod response;

pub use client::*;
