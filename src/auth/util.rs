use thiserror::Error;

pub mod url {
    pub const AUTH_URL  : &str = "https://www.reddit.com/api/v1/authorize"; 
    pub const TOKEN_URL : &str = "https://www.reddit.com/api/v1/access_token";
}

pub(super) mod scope_value {
    pub const IDENTITY_SCOPE        : &str = "identity";
    pub const EDIT_SCOPE            : &str = "edit";
    pub const FLAIR_SCOPE           : &str = "flair";
    pub const HISTORY_SCOPE         : &str = "history";
    pub const MODCONFIG_SCOPE       : &str = "modconfig";
    pub const MODFLAIR_SCOPE        : &str = "modflair";
    pub const MODLOG_SCOPE          : &str = "modlog";
    pub const MODPOSTS_SCOPE        : &str = "modposts";
    pub const MODWIKI_SCOPE         : &str = "modwiki";
    pub const MYSUBREDDITS_SCOPE    : &str = "mysubreddits";
    pub const PRIVATEMESSAGES_SCOPE : &str = "privatemessages";
    pub const READ_SCOPE            : &str = "read";
    pub const REPORT_SCOPE          : &str = "report";
    pub const SAVE_SCOPE            : &str = "save";
    pub const SUBMIT_SCOPE          : &str = "submit";
    pub const SUBSCRIBE_SCOPE       : &str = "subscribe";
    pub const VOTE_SCOPE            : &str = "vote";
    pub const WIKIEDIT              : &str = "wikiedit";
    pub const WIKIREAD              : &str = "wikiread";
}

#[derive(Copy, Clone)]
pub enum Scope {
    Identity,
    Edit,
    Flair,
    History,
    ModConfig,
    ModFlair,
    ModLog,
    ModPosts,
    ModWiki,
    MySubreddits,
    PrivateMessages,
    Read,
    Report,
    Save,
    Submit,
    Subscribe,
    Vote,
    WikiEdit,
    WikiRead
}

pub fn get_scope_value(scope: Scope) -> String {
    match scope {
        Scope::Identity => scope_value::IDENTITY_SCOPE.to_string(),
        Scope::Edit => scope_value::EDIT_SCOPE.to_string(),
        Scope::Flair => scope_value::FLAIR_SCOPE.to_string(),
        Scope::History => scope_value::HISTORY_SCOPE.to_string(),
        Scope::ModConfig => scope_value::MODCONFIG_SCOPE.to_string(),
        Scope::ModFlair => scope_value::MODFLAIR_SCOPE.to_string(),
        Scope::ModLog => scope_value::MODLOG_SCOPE.to_string(),
        Scope::ModPosts => scope_value::MODPOSTS_SCOPE.to_string(),
        Scope::ModWiki => scope_value::MODWIKI_SCOPE.to_string(),
        Scope::MySubreddits => scope_value::MYSUBREDDITS_SCOPE.to_string(),
        Scope::PrivateMessages => scope_value::PRIVATEMESSAGES_SCOPE.to_string(),
        Scope::Read => scope_value::READ_SCOPE.to_string(),
        Scope::Report => scope_value::REPORT_SCOPE.to_string(),
        Scope::Save => scope_value::SAVE_SCOPE.to_string(),
        Scope::Submit => scope_value::SUBMIT_SCOPE.to_string(),
        Scope::Subscribe => scope_value::SUBSCRIBE_SCOPE.to_string(),
        Scope::Vote => scope_value::VOTE_SCOPE.to_string(),
        Scope::WikiEdit => scope_value::WIKIEDIT.to_string(),
        Scope::WikiRead => scope_value::WIKIREAD.to_string()
    }
}

#[derive(Error, Debug)]
pub enum OauthFlowError {
    #[error("Failure: {0}")]
    Failure(String),

    #[error("No refresh token received in Oauth 2.0 flow.")]
    NoRefreshTokenReceived,

    #[error("Error sending client response: {0}")]
    ResponseError(String),

    #[error("State mismatch with Csrf token: {0}, {1}")]
    StateMismatch(String, String),

    #[error("Failed to start TcpListener: {0}")]
    TcpListenerError(String),

    #[error("Error exchanging tokens: {0}")]
    TokenExchangeError(String)
}
