use async_trait::async_trait;
use derive_builder::Builder;
use serde_json::Value;

use super::util::{ Request, RequestBuilder, Error };
use crate::REDDIT_API_URL;
use super::util::RedditClient;

#[derive(Clone, Debug, Default)]
pub enum ListingType {
    #[default]
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

#[derive(Builder, Default)]
#[builder(setter(into, strip_option), build_fn(validate = "Self::validate"))]
pub struct ListingRequest {
    subreddit: String,
    listing_type: ListingType,
    after: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
    g: Option<String>,
    t: Option<SortTime>
}

macro_rules! extract_string_with_default {
    ($a: expr, $b: expr) => {
        $a.as_str().map_or_else(|| { String::new() }, |v| { v.to_string() })
    }
}

macro_rules! extract_string_with_error {
    ($a: expr, $b: expr) => {
        $a.as_str().ok_or(Error::InternalError(format!("Failed to parse to string: {}", $b)))?.to_string()
    }
}

#[async_trait]
impl Request<Listing> for ListingRequest {
    fn get_query_params(&self) -> Vec<(String, String)> {
        let mut query_params: Vec<(String, String)> = Vec::new();

        if let Some(after_str) = &self.after {
            query_params.push(("after".to_string(), after_str.clone()));
        }

        if let Some(before_str) = &self.before {
            query_params.push(("before".to_string(), before_str.clone()));
        }

        let limit_str: String;
        if let Some(limit_u32) = &self.limit {
            limit_str = limit_u32.to_string();
            query_params.push(("limit".to_string(), limit_str.to_string()));
        }

        if let Some(g_str) = &self.g {
            query_params.push(("g".to_string(), g_str.to_string()));
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
            query_params.push(("t".to_string(), t_str.to_string()));
        }
        return query_params;
    }

    fn get_url(&self) -> String {
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
        return format!("{}r/{}/{}", REDDIT_API_URL, self.subreddit, listing_string)
    }

    fn cast_response_structure(value: serde_json::Value) -> Result<Listing, Error> {
        Ok(Listing {
                    after: extract_string_with_default!(value["data"]["after"], "data.after"),
                    before: extract_string_with_default!(value["data"]["before"], "data.before"),
                    posts: Result::<Vec<Post>, Error>::from_iter::<Vec<Result<Post, Error>>>(
                        value["data"]["children"].as_array()
                        .ok_or(Error::InternalError("Can't convert to vector".to_string()))?
                        .into_iter()
                        .map(|v_post| -> Result<Post, Error> {
                            Ok(Post {
                                subreddit: extract_string_with_error!(v_post["data"]["subreddit"], "data.subreddit"),
                                title: extract_string_with_error!(v_post["data"]["title"], "data.title"),
                                selftext: extract_string_with_error!(v_post["data"]["selftext"], "data.selftext"),
                                score: v_post["data"]["score"].as_u64()
                                    .ok_or(Error::InternalError(
                                            format!("Failed to parse to u64: {}", "data.score")))?
                            })
                        })
                        .collect())?
        })
    }

    async fn execute(&self, client: &RedditClient) -> Result<Listing, Error> {
        let res = client.http_client
            .get(&self.get_url())
            .bearer_auth(
                client.oauth_client.access_token.clone().ok_or(
                    Error::InternalError("No access token found in oauth_client.".to_string())
                    )?.secret())
            .query(&self.get_query_params())
            .send()
            .await
            .map_err(|e| { Error::RequestError(format!("Error occurred while sending request: {:?}", e)) })?;

        // TODO: Better error handling.
        let v: Value = serde_json::from_str(&res.text().await.map_err(
                |e| { Error::InternalError(e.to_string()) })?).map_err(
            |e| { Error::InternalError(e.to_string()) }
        )?;

        Ok(Self::cast_response_structure(v)?)
    }
}

impl RequestBuilder<ListingRequest> for ListingRequestBuilder {
    fn validate(&self) -> Result<(), String> {
        if let Some(ListingType::Hot) = self.listing_type {
            if self.g.is_none() {
                return Err("ListingType::Hot must have g parameter".to_string());
            }
        }
        if let Some(ListingType::Top) = self.listing_type {
            if self.t.is_none() {
                return Err("ListingType::Top must have t parameter".to_string());
            }
        }
        if let Some(ListingType::Controversial) = self.listing_type {
            if self.t.is_none() {
                return Err("ListingType::Controversial must have t parameter".to_string());
            }
        }
        Ok(())
    }
} 

#[derive(Default, Debug)]
pub struct Post {
    pub subreddit: String,
    pub title: String,
    pub selftext: String,
    pub score: u64
}

#[derive(Default, Debug)]
pub struct Listing {
    pub after: String,
    pub before: String,
    pub posts: Vec<Post>
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use super::*;
    
    const CLIENT_ID: &str = "nxi3C20O1X1tUYk18hdMvA";
    const REDIRECT_URL: &str = "http://localhost:5555";

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_listing_request() -> Result<()> {

        let mut reddit_client = RedditClient::new(
            CLIENT_ID,
            REDIRECT_URL,
            Some("2303436387124-OQI1VA8iBWjCKvlqPm24NDKJa0M0Sg"),
            "Linux:ravana:cyanide4dinner"
        )?;

        reddit_client.oauth_client.refresh_access_token().await.map_err(|e| { Error::InternalError(e.to_string()) })?;

        // let listing = ListingRequestBuilder::new("rust", ListingType::Hot)
        //     .limit(1)?
        //     .build()
        //     .execute(&reddit_client).await?;
        //
        // println!("{:#?}", listing);

        println!("Access token: \n{:#?}", reddit_client.oauth_client.access_token.ok_or(Error::InternalError("No access token".to_string()))?.secret());

        Ok(())
    }
}
