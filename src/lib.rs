use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthorizationCode,
    ClientId,
    AuthUrl,
    RedirectUrl,
    CsrfToken,
    Scope,
    TokenUrl,
    TokenResponse
};
use url::Url;
use std::{
    fs,
    net::TcpListener,
    io::{BufRead, BufReader, Write}
};

pub mod auth;
pub mod api;

pub const REDDIT_API_URL: &str = "https://oauth.reddit.com/";

const CLIENT_ID: &str = "CO0m-UAASpcd25xiQdi30g";
const AUTH_URL: &str = "https://www.reddit.com/api/v1/authorize"; 
const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
const REDIRECT_URL: &str = "http://localhost:5555"; 

pub fn oauth_process() {
    let client_id = ClientId::new(CLIENT_ID.to_string());
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).expect("Cannot set Auth URL");
    let token_url = TokenUrl::new(TOKEN_URL.to_string()).expect("Cannot set Token URL");
    let client = BasicClient::new(
        client_id,
        None,
        auth_url,
        Some(token_url)
    ).set_redirect_uri(
        RedirectUrl::new(REDIRECT_URL.to_string()).expect("Invalid redirect URL")
    );
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identity".to_string()))
        .add_scope(Scope::new("edit".to_string()))
        .add_scope(Scope::new("flair".to_string()))
        .add_scope(Scope::new("history".to_string()))
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("wikiread".to_string()))
        .add_scope(Scope::new("submit".to_string()))
        .url();

    println!(
        "Open this URL in your browser: \n{}\n",
        authorize_url.to_string()
    );

    let listener = TcpListener::bind("127.0.0.1:5555").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            //let message = "<html><body><h1>Go back to your terminal</h1></body></html>";
            let message = fs::read_to_string("src/auth/oauth-complete.html").expect("Failed to read HTML to string.");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
                );

            stream.write_all(response.as_bytes()).unwrap();

            println!("Reddit returned the following code: \n{}\n", code.secret());
            println!(
                "Reddit returned state: \n{} (expected: `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            let token_res = client.exchange_code(code).request(http_client).unwrap();

            println!("Reddit returned the following token:\n{:?}\n", token_res);
            println!("{}", token_res.access_token().secret());

            break;
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::oauth_process;
//
//     #[test]
//     fn debug_oauth_flow() {
//         oauth_process();
//     }
// }
