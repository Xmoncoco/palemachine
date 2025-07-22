use std::{ thread};
use actix_web::http::header;
use reqwest::Client;
use chrono::Utc;
use crate::db_link;


#[allow(dead_code)]
pub fn downloadvideo(url : &String) {
    let url = url.clone();
    thread::spawn(move ||{
        
    });
}

pub async fn get_image(url: &String, name: &String,ip : &String) {
    let url = url.clone();
    let name = name.clone();
    let ip = ip.clone();
    tokio::spawn(async move {
        println!("c'est url {url}, et le nom {name}");

        if let Some(id) = extract_param(&url, "v") {
            println!("a new image ask with the youtube ID: {}", id);

            if let Ok(youtube_api_key) = std::env::var("YOUTUBE_API_KEY") {
                
                let youtube_api_name_query = format!(
                    "https://www.googleapis.com/youtube/v3/videos?part=snippet&id={}&key={}",
                    id, youtube_api_key
                );
                if let Some(body) = http_get(&youtube_api_name_query).await{
                    let title = get_title_from_json(&body);
                    if let Some(title) = title {
                        let entry: db_link::DbEntry = db_link::DbEntry{
                            url: url.clone(),
                            yt_id: id.clone(),
                            friendly_name : name.clone(),
                            real_name : title.clone(),
                            timestamp : Utc::now().to_rfc3339(),
                            ip : ip.clone()
                        };
                        if let Err(e) = db_link::add_entry(entry) {
                            eprintln!("Failed to add entry to database: {e}");
                        }
                    }
                } else {
                    eprintln!("Failed to fetch video details from YouTube API");
                }
            }else{
                eprintln!("have you set the YOUTUBE_API_KEY env variable?");
            }
        } else {
            println!("No ID found in the URL");
        }
    }).await.unwrap();
}

fn extract_param(url: &str, key: &str) -> Option<String> {
    let key_eq = format!("{}=", key);
    let start = url.find(&key_eq)? + key_eq.len();
    let end = url[start..].find('&').map(|i| start + i).unwrap_or(url.len());
    Some(url[start..end].to_string())
}

pub async fn http_get(url: &str) -> Option<String> {
    let client = Client::new();
    match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => Some(text),
            Err(e) => {
                eprintln!("Erreur lors de la lecture du corps: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de la requête GET: {e}");
            None
        }
    }
}

fn get_title_from_json(json:&str) -> Option<String>{
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
            if let Some(first_item) = items.first() {
                if let Some(snippet) = first_item.get("snippet") {
                    if let Some(title) = snippet.get("title").and_then(|t| t.as_str()) {
                        return Some(title.to_string());
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to parse JSON response");
    }
    None
}
// note sur ce code je l'ai fait a 1h30 le lundi 21 juillet j'ai besoin de sommeil mais pas grave c'est pas en dormant que je pourait implémenter ceci
#[allow(dead_code)]
pub async fn get_thumbnails(api_key: &str, title: &str, friendly_name: &str) {
    let baseurl = "https://api.spotify.com/v1/search?q=";
    let list = [title, friendly_name];

    for element in list {
        let url = format!("{}{}", baseurl, element);

        let mut headers = header::HeaderMap::new();
        let auth_value = format!("Bearer {}", api_key);
        headers.insert(header::AUTHORIZATION, auth_value.parse().unwrap());

        let client = Client::new();

        match client.get(&url)
            .headers(headers.into())
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text){
                        if let Some(items) = json.get("tracks")
                            .and_then(|t| t.get("items"))
                            .and_then(|i| i.get("album"))
                            .and_then(|owari| owari.as_array()){
                                for album in items{
                                    if let Some(name) = album.get("name").and_then(|n| n.as_str()){
                                        println!("{}", name);
                                    }
                                }
                            }else{
                                eprintln!("erreur de structure (j'ai envie de creuver)")
                            }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur de lecture du body : {e}");
                }
            },
            Err(e) => {
                eprintln!("Erreur requête GET : {e}");
            }
        }
    }
}


