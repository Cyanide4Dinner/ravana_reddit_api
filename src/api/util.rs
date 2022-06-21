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
        ) -> Result<Self, RequestError> {
        let mut oauth_client = OauthClient::new(client_id, redirect_url);
        oauth_client.refresh_token = Some(
            RefreshToken::new(refresh_token.to_string())
        );


        Ok(RedditClient {
            oauth_client,
            http_client: HTTPClient::builder().user_agent(user_agent).build()
                .map_err(|e| { RequestError::Failure(e.to_string()) })?
        })
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Error building request: {0}")]
    HTTPRequestBuildError(String),

    #[error("Failure: {0}")]
    Failure(String)
}

#[derive(Error, Debug)]
#[error("Invalid parameters for request builder.")]
pub struct InvalidRequestBuilderParamsError;

#[async_trait]
pub trait Request<S> {
    fn get_filled_builder(&self, client: &RedditClient) -> Result<HTTPRequestBuilder, RequestError>;
    fn construct(&self, client: &RedditClient) -> Result<HTTPRequest, RequestError>;
    async fn send(&self, client: &RedditClient) -> Result<S, RequestError>;
}

pub trait RequestBuilder<R: Clone> {
    fn build(&self) -> R;
}
