use thiserror::Error;
use reqwest::{ Client as HTTPClient, Request as HTTPRequest, RequestBuilder as HTTPRequestBuilder };
use async_trait::async_trait;
use oauth2::RefreshToken;

use crate::auth::oauth::OauthClient;

pub struct RedditClient {
    pub oauth_client: OauthClient,
    pub http_client: HTTPClient
}

impl RedditClient {
    pub fn new(
        client_id: &str,
        redirect_url: &str,
        refresh_token: &str,
        user_agent: &str,
        ) -> Result<Self, Error> {
        let mut oauth_client = OauthClient::new(client_id, redirect_url);
        oauth_client.refresh_token = Some(
            RefreshToken::new(refresh_token.to_string())
        );


        Ok(RedditClient {
            oauth_client,
            http_client: HTTPClient::builder().user_agent(user_agent).build()
                .map_err(|e| { 
                    Error::InternalError(format!("Failed to build HTTP client from builder: {:?}.",
                                                 e)) })?
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    // For errors that occur because of internal fault in the library.
    #[error("Internal error in library: {0}")]
    InternalError(String),

    // For all the requesting sending, response receiving and internet errors that may arise.
    #[error("Error occurred while sending request / receiving response: {0}")]
    RequestError(String),

    // Errors arising to invalid assertions on values or use of library functions or structures by user.
    #[error("Error: {0}")]
    UserError(String)
}

#[async_trait]
pub trait Request<S> {
    fn get_filled_builder(&self, client: &RedditClient) -> Result<HTTPRequestBuilder, Error>;
    fn construct(&self, client: &RedditClient) -> Result<HTTPRequest, Error>;
    async fn send(&self, client: &RedditClient) -> Result<S, Error>;
}

pub trait RequestBuilder<R: Clone> {
    fn build(&self) -> R;
}
