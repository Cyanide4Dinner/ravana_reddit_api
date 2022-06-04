use thiserror::Error;

pub mod Url {
    pub const AUTH_URL  : &str = "https://www.reddit.com/api/v1/authorize"; 
    pub const TOKEN_URL : &str = "https://www.reddit.com/api/v1/access_token";
}

pub(super) mod ScopeValue {
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
        Scope::Identity => ScopeValue::IDENTITY_SCOPE.to_string(),
        Scope::Edit => ScopeValue::EDIT_SCOPE.to_string(),
        Scope::Flair => ScopeValue::FLAIR_SCOPE.to_string(),
        Scope::History => ScopeValue::HISTORY_SCOPE.to_string(),
        Scope::ModConfig => ScopeValue::MODCONFIG_SCOPE.to_string(),
        Scope::ModFlair => ScopeValue::MODFLAIR_SCOPE.to_string(),
        Scope::ModLog => ScopeValue::MODLOG_SCOPE.to_string(),
        Scope::ModPosts => ScopeValue::MODPOSTS_SCOPE.to_string(),
        Scope::ModWiki => ScopeValue::MODWIKI_SCOPE.to_string(),
        Scope::MySubreddits => ScopeValue::MYSUBREDDITS_SCOPE.to_string(),
        Scope::PrivateMessages => ScopeValue::PRIVATEMESSAGES_SCOPE.to_string(),
        Scope::Read => ScopeValue::READ_SCOPE.to_string(),
        Scope::Report => ScopeValue::REPORT_SCOPE.to_string(),
        Scope::Save => ScopeValue::SAVE_SCOPE.to_string(),
        Scope::Submit => ScopeValue::SUBMIT_SCOPE.to_string(),
        Scope::Subscribe => ScopeValue::SUBSCRIBE_SCOPE.to_string(),
        Scope::Vote => ScopeValue::VOTE_SCOPE.to_string(),
        Scope::WikiEdit => ScopeValue::WIKIEDIT.to_string(),
        Scope::WikiRead => ScopeValue::WIKIREAD.to_string()
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
