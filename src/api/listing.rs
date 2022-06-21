use reqwest::{ Request as HTTPRequest,
    RequestBuilder as HTTPRequestBuilder
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use super::util::{ Request, RequestBuilder, RequestError, InvalidRequestBuilderParamsError };
use crate::REDDIT_API_URL;
use super::util::RedditClient;

#[derive(Clone)]
pub enum ListingType {
    Hot,
    New,
    Best,
    Random,
    Rising,
    Top,
    Controversial
}

#[derive(Clone)]
pub enum SortTime {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All
}

#[derive(Clone)]
pub struct ListingRequest {
    subreddit: String,
    listing_type: ListingType,
    after: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
    g: Option<String>,
    t: Option<SortTime>
}

#[async_trait]
impl Request<Listing> for ListingRequest {
    fn get_filled_builder(&self, client: &RedditClient) -> Result<HTTPRequestBuilder, RequestError> {
        let mut query_params: Vec<(&str, &str)> = Vec::new();

        if let Some(after_str) = &self.after {
            query_params.push(("after", &after_str));
        }
        
        if let Some(before_str) = &self.before {
            query_params.push(("after", &before_str));
        }

        let limit_str: String;
        if let Some(limit_u32) = &self.limit {
            limit_str = limit_u32.to_string();
            query_params.push(("limit", &limit_str));
        }

        if let Some(g_str) = &self.g {
            query_params.push(("g", &g_str));
        }

        if let Some(t_sort_time) = &self.t {
            let t_str: &str;
            match t_sort_time {
                SortTime::Hour => { t_str = "hour"; },
                SortTime::Day => { t_str = "day" },
                SortTime::Week => { t_str = "week" },
                SortTime::Month => { t_str = "month"},
                SortTime::Year => { t_str = "year" },
                SortTime::All => { t_str = "all" }
            }
            query_params.push(("t", t_str));
        }

        let listing_string: &str;
        match self.listing_type {
            ListingType::Hot => { listing_string = "hot" },
            ListingType::Top => { listing_string = "top" },
            ListingType::New => { listing_string = "new" },
            ListingType::Best => { listing_string = "best" },
            ListingType::Rising => { listing_string = "rising" },
            ListingType::Controversial => { listing_string = "controversial" },
            ListingType::Random => { listing_string = "random" }
        }

        Ok(client.http_client
            .get(format!("{}r/{}/{}", REDDIT_API_URL, self.subreddit, listing_string))
            .bearer_auth(
                client.oauth_client.access_token.clone().ok_or(
                    RequestError::HTTPRequestBuildError("No access token found.".to_string())
                    )?.secret())
            .query(&query_params))
    }

    fn construct(&self, client: &RedditClient) -> Result<HTTPRequest, RequestError> {
        self.get_filled_builder(client)?
            .build()
            .map_err(|e| { RequestError::HTTPRequestBuildError(e.to_string()) }) 
    }

    async fn send(&self, client: &RedditClient) -> Result<Listing, RequestError> {
        let res = self.get_filled_builder(client)?
            .send()
            .await
            .map_err(|e| { RequestError::Failure(e.to_string()) })?;

        // TODO: Better error handling.
        let v: Value = serde_json::from_str(&res.text().await.map_err(|e| { RequestError::Failure(e.to_string()) })?).map_err(
            |e| { RequestError::Failure(e.to_string()) }
        )?;

        let listing = Listing {
            after: v["data"]["after"].as_str().ok_or(RequestError::Failure("Failed".to_string()))?.to_string(),
            before: v["data"]["before"].to_string(),
            posts: v["data"]["children"].as_array()
                .ok_or(RequestError::Failure("Can't convert to vector".to_string()))?
                .into_iter()
                .map(|v_post| { 
                    Post { 
                        subreddit: v_post["data"]["subreddit"].to_string(),
                        title: v_post["data"]["title"].to_string(),
                        selftext: v_post["data"]["selftext"].to_string() 
                    } 
                })
                .collect()
        };

        // println!("{}", res.text().await.map_err(|e| { RequestError::Failure(e.to_string()) })?);
        println!("{:?}", listing);

        Ok(listing)
    }
}

#[derive(Default, Debug)]
pub struct Post {
    pub subreddit: String,
    pub title: String,
    pub selftext: String
}

#[derive(Default, Debug)]
pub struct Listing {
    pub after: String,
    pub before: String,
    pub posts: Vec<Post>
}

struct ListingRequestBuilder {
    req: ListingRequest
}

macro_rules! assert_listing_type {
    ($a: expr, $( $lt: path ),+) => {
        match $a {
            $($lt => {  },)+
            _ => { return Err(InvalidRequestBuilderParamsError); }
        };
    }
}

impl ListingRequestBuilder {
    fn new(subreddit: &str, listing_type: ListingType) -> Self {
        ListingRequestBuilder {
            req: ListingRequest {
                subreddit: subreddit.to_string(),
                listing_type,
                after: None,
                before: None,
                limit: None,
                g: None,
                t: None
            }
        }
    }

    fn after(mut self, after: String) -> Result<Self, InvalidRequestBuilderParamsError> {
        assert_listing_type!(self.req.listing_type,
                             ListingType::Top,
                             ListingType::New,
                             ListingType::Best,
                             ListingType::Rising,
                             ListingType::Controversial
                             );
        self.req.after = Some(after);
        Ok(self)
    }

    fn before(mut self, before: String) -> Result<Self, InvalidRequestBuilderParamsError> {
        assert_listing_type!(self.req.listing_type,
                             ListingType::Top,
                             ListingType::New,
                             ListingType::Best,
                             ListingType::Rising,
                             ListingType::Controversial
                             );
        self.req.before = Some(before);
        Ok(self)
    }

    fn limit(mut self, limit: u32) -> Result<Self, InvalidRequestBuilderParamsError> {
        assert_listing_type!(self.req.listing_type,
                             ListingType::Top,
                             ListingType::New,
                             ListingType::Best,
                             ListingType::Rising,
                             ListingType::Controversial
                             );
        self.req.limit = Some(limit);
        Ok(self)
    }

    fn g(mut self, g: String) -> Result<Self, InvalidRequestBuilderParamsError> {
        assert_listing_type!(self.req.listing_type,
                             ListingType::Best
                             );
        self.req.g = Some(g);
        Ok(self)
    }

    fn t(mut self, sort_time: SortTime) -> Result<Self, InvalidRequestBuilderParamsError> {
        assert_listing_type!(self.req.listing_type,
                             ListingType::Top,
                             ListingType::Controversial
                             );
        self.req.t = Some(sort_time);
        Ok(self)
    }
}

impl RequestBuilder<ListingRequest> for ListingRequestBuilder {
    fn build(&self) -> ListingRequest {
        self.req.clone()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use super::*;
    use crate::auth::*;
    use oauth2::RefreshToken;
    use oauth2::AccessToken;
    use reqwest::Client as HTTPClient;
    use tokio::task::spawn_blocking;
    
    const CLIENT_ID: &str = "CO0m-UAASpcd25xiQdi30g";
    const REDIRECT_URL: &str = "http://localhost:5555";

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_listing_request() -> Result<()> {

        let mut reddit_client = RedditClient::new(
            CLIENT_ID,
            REDIRECT_URL,
            "1760391933139-mCRQ75-BxXVHYVS6sQdJhSyDRdX8JA",
            "Linux:ravana:cyanide4dinner"
        )?;

        reddit_client.oauth_client.refresh_access_token().await.map_err(|e| { RequestError::Failure(e.to_string()) })?;

        ListingRequestBuilder::new("rust", ListingType::New)
            .limit(1)?
            .build()
            .send(&reddit_client).await?;

        println!("Access token: {}", reddit_client.oauth_client.access_token.ok_or(RequestError::Failure("No access token".to_string()))?.secret());

        Ok(())
    }
}
