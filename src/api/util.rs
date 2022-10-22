use thiserror::Error;
use reqwest::Client as HTTPClient;
use async_trait::async_trait;
use oauth2::RefreshToken;

use crate::auth::Scope;
use crate::auth::oauth::OauthClient;

pub struct RedditClient {
    pub oauth_client: OauthClient,
    pub http_client: HTTPClient
}

impl RedditClient {
    pub fn new(
        client_id: &str,
        redirect_url: &str,
        refresh_token: Option<&str>,
        user_agent: &str,
        ) -> Result<Self, Error> {
        let mut oauth_client = OauthClient::new(client_id, redirect_url);
        if let Some(tok) = refresh_token {
            oauth_client.refresh_token = Some(RefreshToken::new(tok.to_string()));
        } else {
            oauth_client.refresh_token = None;
        }

        Ok(RedditClient {
            oauth_client,
            http_client: HTTPClient::builder().user_agent(user_agent).build()
                .map_err(|e| { 
                    Error::InternalError(format!("Failed to build HTTP client from builder: {:?}.",
                                                 e)) })?
        })
    }

    pub async fn refresh_token(&mut self) -> Result<(), Error> {
        self.oauth_client.refresh_access_token().await.map_err(|e| { Error::OauthError(e.to_string()) })?;
        Ok(())
    }

    pub fn oauth_url(&self, scopes: Vec<Scope>) -> (url::Url, oauth2::CsrfToken) {
        self.oauth_client.oauth_url(scopes)
    }

    pub async fn oauth_flow(&mut self, csrf: oauth2::CsrfToken, success_message: String) -> Result<(), Error> {
        self.oauth_client.oauth_flow(csrf, success_message).await.map_err(|e| { 
            Error::OauthError(format!("Oauth flow error: {:?}", e)) })
    }

}

#[derive(Error, Debug)]
pub enum Error {
    // For all errors that arose because of Oauth.
    #[error("Oauth related error: {0}")]
    OauthError(String),

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
pub trait Request<S: Default> {
    fn get_query_params(&self) -> Vec<(String, String)>;
    fn get_url(&self) -> String;
    fn cast_response_structure(value: serde_json::Value) -> Result<S, Error>;
    async fn execute(&self, client: &RedditClient) -> Result<S, Error>;
}

pub trait RequestBuilder<R> {
    fn validate(&self) -> Result<(), String>;
}
