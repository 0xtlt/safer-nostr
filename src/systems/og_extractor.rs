use reqwest::Client;
use select::document::Document;
use select::predicate::{Attr, Name};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct OgInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Error)]
pub enum OgExtractorError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

pub async fn og_extractor(url: &str) -> Result<OgInfo, OgExtractorError> {
    println!("Extracting OG info from {url}");
    let client = Client::new();

    // Create fake user agent to bypass anti-scraping measures
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99 Safari/537.36".to_string();
    let res = client
        .get(url)
        .header("User-Agent", user_agent)
        .send()
        .await?;

    let mut og_info = OgInfo {
        title: None,
        description: None,
        image: None,
    };

    if res.status().is_success() {
        let body = res.text().await?;

        let document = Document::from(body.as_str());

        og_info.title = {
            let og_title = document
                .find(Attr("property", "og:title"))
                .next()
                .map(|node| node.attr("content").unwrap_or_default().to_string());

            if og_title.is_none() {
                document.find(Name("title")).next().map(|node| node.text())
            } else {
                og_title
            }
        };

        og_info.description = {
            let og_description = document
                .find(Attr("property", "og:description"))
                .next()
                .map(|node| node.attr("content").unwrap_or_default().to_string());

            if og_description.is_none() {
                document
                    .find(Name("meta"))
                    .find(|node| node.attr("name") == Some("description"))
                    .map(|node| node.attr("content").unwrap_or_default().to_string())
            } else {
                og_description
            }
        };

        og_info.image = {
            let og_image = document
                .find(Attr("property", "og:image"))
                .next()
                .map(|node| node.attr("content").unwrap_or_default().to_string());

            if og_image.is_none() {
                document
                    .find(Name("img"))
                    .find(|node| node.attr("src").is_some())
                    .map(|node| node.attr("src").unwrap_or_default().to_string())
            } else {
                og_image
            }
        };
    } else {
        println!("Error: {}", res.status());
    }

    Ok(og_info)
}
