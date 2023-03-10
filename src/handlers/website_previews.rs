use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Info {
    url: String,
}

pub async fn get(info: web::Query<Info>, data: web::Data<crate::WebStates>) -> impl Responder {
    let cache_key = format!("og:{}", info.url);

    let og = match data.cache.to_owned().get_str(&cache_key).await {
        Ok(og) => og,
        Err(_) => {
            let og = crate::systems::og_extractor::og_extractor(&info.url)
                .await
                .unwrap();

            let og_str = serde_json::to_string(&og).unwrap();

            data.cache
                .to_owned()
                .set_str(&cache_key, &og_str, crate::ENV_CONFIG.cache_ttl_webpreview)
                .await
                .unwrap();
            og_str
        }
    };

    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(og)
}
