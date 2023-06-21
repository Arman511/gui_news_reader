#[cfg(feature = "async")]
use reqwest::{Client, Method};

use serde::Deserialize;
use url::Url;
const BASE_URL: &str = "https://newsdata.io/api/1/news";

#[derive(thiserror::Error, Debug)]
pub enum NewsAPIError {
    #[error("Failed fetching articles")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed Conversion of response to string")]
    FailedResponseToString(#[from] std::io::Error),
    #[error("Article Parsing Failed")]
    FailedParsing(#[from] ureq::serde_json::Error),
    #[error("Failed to parse url")]
    URLParsing(#[from] url::ParseError),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
    #[error("Failed async fetch")]
    #[cfg(feature = "async")]
    AsyncRequestFailed(#[from] reqwest::Error),
}

#[derive(Deserialize, Debug)]
pub struct NewsAPIResponse {
    status: String,
    pub results: Vec<Article>,
    code: Option<String>,
}

impl NewsAPIResponse {
    pub fn articles(&self) -> &Vec<Article> {
        &self.results
    }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    title: String,
    link: String,
    description: Option<String>,
}

impl Article {
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_link(&self) -> &str {
        &self.link
    }

    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}
pub enum Category {
    TopHeadLines,
}
pub enum Country {
    GB,
    US,
}
pub enum Languages {
    EN,
}
pub struct NewsAPI {
    api_key: String,
    category: Category,
    country: Country,
    language: Languages,
}

impl NewsAPI {
    pub fn new(api_key: &str) -> NewsAPI {
        NewsAPI {
            api_key: api_key.to_string(),
            category: Category::TopHeadLines,
            country: Country::GB,
            language: Languages::EN,
        }
    }

    pub fn category(&mut self, category: Category) -> &mut NewsAPI {
        self.category = category;
        self
    }
    pub fn country(&mut self, country: Country) -> &mut NewsAPI {
        self.country = country;
        self
    }
    pub fn language(&mut self, language: Languages) -> &mut NewsAPI {
        self.language = language;
        self
    }
    fn prepare_url(&self) -> Result<String, NewsAPIError> {
        let base_url = Url::parse(BASE_URL)?;

        let fmt_url = format!(
            "{}?apikey={}&language={}&country={}&category={}",
            base_url,
            self.api_key,
            self.language.to_string(),
            self.country.to_string(),
            self.category.to_string(),
        );

        Ok(fmt_url)
    }

    pub fn fetch(&self) -> Result<NewsAPIResponse, NewsAPIError> {
        let url = self.prepare_url()?;
        let req = ureq::get(&url);
        let response: NewsAPIResponse = req.call()?.into_json()?;
        match response.status.as_str() {
            "success" => return Ok(response),
            _ => return Err(map_response_err(response.code)),
        }
    }

    #[cfg(feature = "async")]
    pub async fn fetch_async(&self) -> Result<NewsAPIResponse, NewsAPIError> {
        let url = self.prepare_url()?;
        let client = Client::new();
        let req = client
            .request(Method::GET, url)
            .build()
            .map_err(|e| NewsAPIError::AsyncRequestFailed(e))?;

        let response = client
            .execute(req)
            .await?
            .text()
            .await
            .map_err(|e| NewsAPIError::AsyncRequestFailed(e))?;
        let news_response: NewsAPIResponse = serde_json::from_str(&response)
        .map_err(NewsAPIError::FailedParsing)?;
        match news_response.status.as_str() {
            "success" => return Ok(news_response),
            _ => return Err(map_response_err(news_response.code)),
        }
    }
}

fn map_response_err(code: Option<String>) -> NewsAPIError {
    if let Some(code) = code {
        match code.as_str() {
            "apiKeyDisabled" => NewsAPIError::BadRequest("Your API key has been disabled"),
            _ => NewsAPIError::BadRequest("Unknown error 1"),
        }
    } else {
        NewsAPIError::BadRequest("Unknown error 2")
    }
}

impl ToString for Category {
    fn to_string(&self) -> String {
        match self {
            Self::TopHeadLines => "top".to_owned(),
        }
    }
}

impl ToString for Country {
    fn to_string(&self) -> String {
        match self {
            Country::GB => "gb".to_owned(),
            Country::US => "us".to_owned(),
        }
    }
}
impl ToString for Languages {
    fn to_string(&self) -> String {
        match self {
            Languages::EN => "en".to_owned(),
        }
    }
}
