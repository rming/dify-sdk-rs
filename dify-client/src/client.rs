//! This module contains the implementation of the Dify client.
//!
//! The `client` module provides a `Client` struct that represents a client for interacting with the Dify API.
//! It also includes a `Config` struct that holds the configuration for the client.
//! The client supports creating requests, executing them, and returning the response.
//! Additionally, it provides methods for creating form requests and handling multipart data.
//!
//! # Examples
//!
//! Creating a new client with default configuration:
//!
//! ```rust
//! use dify_client::client::Client;
//!
//! let client = Client::new("https://api.dify.ai", "API_KEY");
//! ```
//!
//! Creating a new client with custom configuration:
//!
//! ```rust
//! use dify_client::client::{Client, Config};
//! use std::time::Duration;
//!
//! let config = Config {
//!     base_url: "https://api.dify.ai".into(),
//!     api_key: "API_KEY".into(),
//!     timeout: Duration::from_secs(30),
//! };
//!
//! let client = Client::new_with_config(config);
//! ```
use super::{
    api::Api,
    http::{header, multipart, Method, Request, Response},
};
use anyhow::{bail, Result as AnyResult};
use std::{sync::Arc, time::Duration};

#[derive(Clone, Debug)]
/// The configuration for the Dify client.
pub struct Config {
    /// The base URL of the Dify API.
    pub base_url: String,
    /// The API key for the Dify API.
    pub api_key: String,
    /// The timeout for the client requests.
    pub timeout: Duration,
}

/// Implements the default configuration for the client.
impl Default for Config {
    /// Returns a new instance of `ClientConfig` with default values.
    fn default() -> Self {
        Self {
            base_url: "https://api.dify.ai".into(),
            api_key: "API_KEY".into(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// The `Client` struct represents a client for interacting with the Dify API.
#[derive(Clone, Debug)]
pub struct Client {
    /// The configuration for the client.
    pub config: Arc<Config>,
    /// The HTTP client for sending requests.
    http_client: reqwest::Client,
}

/// The `Client` struct represents a client for interacting with the Dify API.
impl Client {
    /// Creates a new `Client` instance with the specified base URL and API key.
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Dify API.
    /// * `api_key` - The API key for authentication.
    ///
    /// # Returns
    /// A new `Client` instance.
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self::new_with_config(Config {
            base_url: base_url.into(),
            api_key: api_key.into(),
            ..Config::default()
        })
    }

    /// Creates a new `Client` instance with the specified configuration.
    ///
    /// # Arguments
    /// * `c` - The configuration for the client.
    ///
    /// # Returns
    /// A new `Client` instance.
    pub fn new_with_config(mut c: Config) -> Self {
        // format the base URL
        c.base_url = c.base_url.trim_end_matches("/").into();
        // build the http client
        let mut builder = reqwest::ClientBuilder::new();
        if !c.timeout.is_zero() {
            builder = builder.timeout(c.timeout);
        }
        let http_client = builder
            .default_headers(Self::default_headers(&c))
            .build()
            .expect("Failed to create http client");

        Self {
            config: Arc::new(c),
            http_client,
        }
    }

    /// Returns the default headers for the client.
    ///
    /// # Arguments
    /// * `c` - The configuration for the client.
    ///
    /// # Returns
    /// The default headers for the client.
    fn default_headers(c: &Config) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );

        let auth = format!("Bearer {}", c.api_key);
        let mut bearer_auth = header::HeaderValue::from_str(&auth).unwrap();
        bearer_auth.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, bearer_auth);
        headers
    }

    /// Returns the API for the client.
    /// The API provides methods for interacting with the Dify API.
    ///
    /// # Returns
    /// The API for the client.
    pub fn api(&self) -> Api {
        Api::new(self)
    }

    /// Creates a request with the specified URL, method, and data.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    /// * `method` - The HTTP method for the request.
    /// * `data` - The data for the request.
    ///
    /// # Returns
    /// A `Result` containing the request or an error.
    ///
    /// # Errors
    /// Returns an error if the request cannot be created.
    ///
    /// # Panics
    /// Panics if the method is not supported.
    pub(crate) fn create_request<T>(
        &self,
        url: String,
        method: Method,
        data: T,
    ) -> AnyResult<Request>
    where
        T: serde::Serialize,
    {
        match method {
            Method::POST => {
                let r = self.http_client.post(url).json(&data).build()?;
                Ok(r)
            }
            Method::GET => {
                let r = self.http_client.get(url).query(&data).build()?;
                Ok(r)
            }
            Method::DELETE => {
                let r = self.http_client.delete(url).json(&data).build()?;
                Ok(r)
            }
            _ => bail!("Method not supported"),
        }
    }

    /// Creates a form request with the specified URL and data.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    /// * `data` - The data for the request.
    ///
    /// # Returns
    /// A `Result` containing the request or an error.
    pub(crate) fn create_multipart_request(
        &self,
        url: String,
        form_data: multipart::Form,
    ) -> AnyResult<Request> {
        let r = self.http_client.post(url).multipart(form_data).build()?;
        Ok(r)
    }

    /// Executes the specified request and returns the response.
    ///
    /// # Arguments
    /// * `request` - The request to execute.
    ///
    /// # Returns
    /// A `Result` containing the response or an error.
    pub(crate) async fn execute(&self, request: Request) -> AnyResult<Response> {
        self.http_client.execute(request).await.map_err(Into::into)
    }
}
