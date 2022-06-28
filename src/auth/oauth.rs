use oauth2::{
    AccessToken,
    AuthorizationCode,
    AuthUrl,
    basic::BasicClient,
    ClientId,
    CsrfToken,
    reqwest::async_http_client,
    reqwest::http_client,
    RefreshToken,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl
};
use url::Url;
use std::{
    net::TcpListener,
    io::{ BufRead, BufReader, Write }
};

use super::{ get_scope_value, Scope  as RedditScope, OauthFlowError, Url::{ AUTH_URL, TOKEN_URL } };

pub struct OauthClient {
    client: BasicClient,
    pub access_token: Option<AccessToken>,
    pub refresh_token: Option<RefreshToken>,
}

impl OauthClient {
    pub fn new(
        client_id: &str,
        redirect_url: &str,
    ) -> Self {
        OauthClient {
            client: BasicClient::new(
                        ClientId::new(client_id.to_string()),
                        None,
                        AuthUrl::new(AUTH_URL.to_string()).expect("Cannot set Auth URL."),
                        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("Cannot set Token URL"))
                    ).set_redirect_uri(
                        RedirectUrl::new(redirect_url.to_string()).expect("Invalid redirect URL")
            ),
            access_token: None,
            refresh_token: None
        }
    }

    pub fn oauth_url(&self, scopes: Vec<RedditScope>) 
        -> (url::Url, oauth2::CsrfToken) {
        let mut auth_req = self.client.authorize_url(CsrfToken::new_random);
        for scope in scopes.iter() {
            auth_req = auth_req.add_scope(Scope::new(get_scope_value(*scope)));
        }
        auth_req = auth_req.add_extra_param("duration", "permanent");
        auth_req.url()
    }

    pub async fn oauth_flow(&mut self,
                      csrf: oauth2::CsrfToken,
                      success_message: String) -> Result<(), OauthFlowError> {
        let redirect_url: &Url = self.client.redirect_url().
            ok_or(OauthFlowError::Failure("No redirect_url set for client.".to_string()))?.url(); 

        let tcp_red_url = 
                redirect_url.host_str()
                    .ok_or(OauthFlowError::TcpListenerError("Cannot get host.".to_string()))?.to_string()
                + ":" +
                &redirect_url.port()
                    .ok_or(OauthFlowError::TcpListenerError("Cannot get port".to_string()))?.to_string();

        let listener = TcpListener::bind(tcp_red_url).map_err(|e| { 
            OauthFlowError::TcpListenerError(e.to_string())
        })?;
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let code: AuthorizationCode;
                let state: CsrfToken;
                {
                    let mut reader = BufReader::new(&stream);

                    let mut request_line = String::new();
                    reader.read_line(&mut request_line)
                        .map_err(|e| { OauthFlowError::Failure(e.to_string()) })?;

                    let state_url_part = request_line.split_whitespace().nth(1).unwrap();
                    let url = Url::parse(&("http://localhost".to_string() + state_url_part))
                        .map_err(|e| { OauthFlowError::Failure(e.to_string()) })?;

                    let code_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "code"
                        })
                        .ok_or(OauthFlowError::Failure("Failed to get code_pair".to_string()))?;

                    let (_, value) = code_pair;
                    code = AuthorizationCode::new(value.into_owned());

                    let state_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "state"
                        })
                        .ok_or(OauthFlowError::Failure("Failed to get state_pair".to_string()))?;

                    let (_, value) = state_pair;
                    state = CsrfToken::new(value.into_owned());
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    success_message.len(),
                    success_message
                    );

                stream.write_all(response.as_bytes())
                    .map_err(|e| { OauthFlowError::ResponseError(e.to_string()) })?;

                if !state.secret().clone().eq(csrf.secret()) {
                    return Err(OauthFlowError::StateMismatch(state.secret().clone(), csrf.secret().clone()))
                }

                let token_response = self.client.exchange_code(code).request_async(async_http_client).await
                    .map_err(|e| { OauthFlowError::TokenExchangeError(e.to_string()) })?;

                self.access_token = Some(token_response.access_token().clone());
                self.refresh_token = Some(token_response.refresh_token()
                    .ok_or(OauthFlowError::NoRefreshTokenReceived)?.clone());

                return Ok(());
            }
        }
        Err(OauthFlowError::Failure("Failed to complete Oauth 2.0 flow".to_string()))
    }

    pub async fn refresh_access_token(&mut self) 
            -> Result<(), OauthFlowError> {
        if let Some(refresh_token) = &self.refresh_token {
            let token_response = self.client.exchange_refresh_token(&refresh_token)
                .request_async(async_http_client)
                .await
                .map_err(|e| { OauthFlowError::TokenExchangeError(e.to_string()) })?;
            self.access_token = Some(token_response.access_token().clone());
            Ok(())
        } else {
            Err(OauthFlowError::Failure("No refresh token in client.".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use super::{ http_client, RedditScope, OauthFlowError };
    use super::OauthClient;

    const CLIENT_ID: &str = "CO0m-UAASpcd25xiQdi30g";
    const REDIRECT_URL: &str = "http://localhost:5555";

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn debug_oauth_flow() -> Result<()> {
        let mut client = OauthClient::new(CLIENT_ID, REDIRECT_URL);
        let (auth_url, csrf_tok) = client.oauth_url(vec!(RedditScope::Read));
        println!("Go to URL: {}", auth_url);
        client.oauth_flow(csrf_tok, "<html><body><h1>Success</h1></body></html>".to_string()).await?;
        println!("Received access token: {}, refresh token: {}",
                 client.access_token.ok_or(OauthFlowError::NoRefreshTokenReceived)?.secret(),
                 client.refresh_token.ok_or(OauthFlowError::NoRefreshTokenReceived)?.secret()
        );
        Ok(())

        // let client = get_oauth_client(CLIENT_ID, REDIRECT_URL);
        // let (auth_url, csrf_tok) = get_oauth_url(
        //     &client,
        //     vec!(RedditScope::Read)
        // );
        //
        // println!("Go to URL: {}", auth_url);
        // let (access_tok, refresh_tok) = oauth_flow(&client, csrf_tok, "<html><body><h1>Success</h1></body></html>".to_string())?;
        // println!("Received access token: {}, refresh token: {}", access_tok.secret(), refresh_tok.secret());
        // Ok(())
    }

    // #[test]
    // #[ignore]
    // fn debug_exchange_refresh_token() -> Result<()> {
    //     let refresh_token = oauth2::RefreshToken::new("166788405649-KAGh3f8GQr6_BQdiHZj2Eys8viPHdQ".to_string());
    //     let client = get_oauth_client(CLIENT_ID, REDIRECT_URL);
    //     let token_response = client.exchange_refresh_token(&refresh_token).request(http_client)?;
    //     println!("New access token got: {}", token_response.access_token().secret());
    //     Ok(())
    // }
}
