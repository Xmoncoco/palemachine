use std::{ fs::{self, File}, process::Command};
use actix_web::http::header;
use reqwest::Client;
use chrono::Utc;
use crate::{db_link, db_link::add_entry };
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};

use toml;


#[derive(Deserialize)]
struct SpotifyToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
#[derive(Debug, Deserialize)]
struct SpotifyThumbnail {
    height: u32,
    url: String,
    width: u32,
}

#[derive(Debug, Serialize)]
pub struct SimpleSpotifyThumbnail{
    music_name: String,
    uri: String,
}


pub async fn get_image(url: &String, name: &String,ip : &String) -> Vec<SimpleSpotifyThumbnail> {
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
                            let _ = add_entry(entry);
                            let get_token = async{get_spotify_token().await};
                            let token = get_token.await;
                             return get_thumbnails(&token, &title, &name).await;

                        }
                    } else {
                        eprintln!("Failed to fetch video details from YouTube API");
                    }
                }else{
                    eprintln!("have you set the YOUTUBE_API_KEY env variable?");
                }
            } else if let Some(id) = extract_param(&url, "list") {
                println!("a new image ask with the playlist ID: {}", id);

                if let Ok(youtube_api_key) = std::env::var("YOUTUBE_API_KEY") {
                    
                    let youtube_api_name_query = format!(
                        "https://www.googleapis.com/youtube/v3/playlists?part=snippet&id={}&key={}",
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
                            let _ = add_entry(entry);
                            let get_token = async{get_spotify_token().await};
                            let token = get_token.await;
                             return get_thumbnails(&token, &title, &name).await;

                        }
                    } else {
                        eprintln!("Failed to fetch playlist details from YouTube API");
                    }
                }else{
                    eprintln!("have you set the YOUTUBE_API_KEY env variable?");
                }
            } else {
                println!("No ID found in the URL");
            }
        return Vec::<SimpleSpotifyThumbnail>::new();
    }).await.unwrap()
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
// note sur ce code je l'ai fait a 1h30 le lundi 21 juillet j'ai besoin de sommeil mais pas grave c'est pas en dormant que je pourait implémenter ceci ok j'ai fait pire le 25 juillet où je code a 3h du matin

pub async fn get_thumbnails(api_key: &str, title: &str, friendly_name: &str) -> Vec<SimpleSpotifyThumbnail> {
    let baseurl = "https://api.spotify.com/v1/search?q=";
    let list = [title  , friendly_name ]; //set as comment for testing purposes

    let mut image_track_list: Vec<SimpleSpotifyThumbnail> = Vec::new();

    for element in list {
        let url = format!("{}{}&type=album", baseurl, element);

        let mut headers = header::HeaderMap::new();
        let auth_value = format!("Bearer {}", api_key);
        headers.insert(header::AUTHORIZATION, auth_value.parse().unwrap());

        let client = Client::new();

        let response = client.get(&url).headers(headers.into()).send().await;
        if let Ok(resp) = response {
            if let Ok(text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(items) = json.get("albums")
                        .and_then(|t| t.get("items"))
                        .and_then(|i| i.as_array())
                    {
                        for album in items {
                            if let (Some(name), Some(image_list)) = (
                                album.get("name").and_then(|n| n.as_str()),
                                album.get("images").and_then(|i| i.as_array()),
                            ) {
                                if let Some(image_json) = image_list.get(0) {
                                    if let Ok(image_serde) = serde_json::from_value::<SpotifyThumbnail>(image_json.clone()) {
                                        if cfg!(debug_assertions){
                                            println!("{}",image_serde.height);
                                            println!("{}", image_serde.width);
                                        }
                                        let element = SimpleSpotifyThumbnail {
                                            music_name: name.to_string(),
                                            uri: image_serde.url,
                                        };
                                        image_track_list.push(element);
                                    }
                                }
                            }
                        }
                    } else {
                        eprintln!("erreur de structure (j'ai envie de creuver)");
                    }
                } else {
                    eprintln!("Erreur parsing JSON");
                }
            } else {
                eprintln!("Erreur de lecture du body");
            }
        } else {
            eprintln!("Erreur requête GET");
        }
    }

    image_track_list
}

pub async fn get_spotify_token() -> String {
    let client_id = std::env::var("SPOTIFY_CLIENT").unwrap_or_default();
    let client_secret = std::env::var("SPOTIFY_SECRET").unwrap_or_default();
    let baseurl = "https://accounts.spotify.com/api/token";

    let creds = format!("{}:{}", client_id, client_secret);
    let auth = format!("Basic {}", general_purpose::STANDARD.encode(creds));

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, auth.parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());

    let client = Client::new();
    let res = client
        .post(baseurl)
        .headers(headers.into())
        .body("grant_type=client_credentials")
        .send()
        .await;

    let res =match res {
        Ok(response) => response,
        Err(e) => {
            println!("Error sending request:{}", e);
            return "".to_string();
        }
    };

    let token : SpotifyToken=res.json().await.expect("Failed to parse response");
    if cfg!(debug_assertions){
        println!("{} {}",token.token_type,token.expires_in)
    }
    
    token.access_token
}

pub async fn send_download(url: &str, name: &str, image: &str) {
    let url = url.to_string();
    let name = name.to_string();
    let image = image.to_string();

    tokio::task::spawn_blocking(move || {
        // Lecture propre de la config TOML
        let configfile = fs::read_to_string("config.toml")
            .expect("config.toml manquant !");
        let config: toml::Value = toml::from_str(&configfile)
            .expect("Erreur de parsing de config.toml");
        let path = config
            .get("path")
            .and_then(|v| v.as_str())
            .expect("Champ 'path' manquant ou mal formé dans config.toml");
        let pythonpath = "./venv/bin/python3";
        let script_path = "./downloader"; // Extension .py explicite

        // On passe l'URL complète au script Python pour qu'il gère les playlists
        let output_file = format!("{}", name);
        let status = Command::new(pythonpath)
            .arg(script_path)
            .arg(&url)
            .arg(&output_file)
            .arg(path)
            .status()
            .expect("Erreur lors du lancement du script Python");
        if status.success() {
            println!("✅ Script Python exécuté avec succès !");
        } else {
            eprintln!("❌ Échec du script Python (code: {:?})", status.code());
        }

        // Téléchargement de l'image
        match download_image(&image, path,&name) {
            Ok(_) => {
                println!("Le processus de téléchargement de l'image est terminé. Vérifiez votre répertoire !");
            }
            Err(e) => {
                eprintln!("❌ Échec du téléchargement ou de l'écriture du fichier image : {}", e);
            }
        }
    })
    .await
    .expect("Erreur lors de l'exécution du spawn_blocking");
}

fn download_image(url: &str, output_path: &str,name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Démarrage du téléchargement depuis : {}", url);
    
    let playlist_check = format!("{}/{}", output_path, name);
    let is_playlist = std::path::Path::new(&playlist_check).is_dir();
    
    let real_path;
    let script_target;
    
    if is_playlist {
        real_path = format!("{}/cover.jpg", playlist_check);
        script_target = playlist_check;
    } else {
        real_path = format!("{}/{}.jpg", output_path, name);
        script_target = output_path.to_string();
    }

    // 1. Créer un client reqwest bloquant et effectuer la requête GET
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(url)
        .send()?            // Envoie la requête
        .error_for_status()?; // Vérifie si la réponse HTTP est un succès (2xx)

    // 2. Créer le fichier de destination local
    let mut file = File::create(&real_path)?;

    // 3. Copier directement le corps de la réponse dans le fichier
    // La méthode copy_to() transfère efficacement les données chunk par chunk.
    let bytes_written = response.copy_to(&mut file)?;

    println!("--------------------------------------------------");
    println!("✅ Succès : {} octets écrits dans le fichier '{}'.", bytes_written, real_path);
    let status = Command::new(r"./bambam_morigatsu_chuapo.sh")
        .arg(script_target)
        .status()
        .expect("Zzz");
    if status.success(){
        println!("ok")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_param() {
        let url_v = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert_eq!(extract_param(url_v, "v"), Some("dQw4w9WgXcQ".to_string()));

        let url_list = "https://www.youtube.com/playlist?list=PL123456789";
        assert_eq!(extract_param(url_list, "list"), Some("PL123456789".to_string()));

        let url_mid = "https://example.com?v=123&other=456";
        assert_eq!(extract_param(url_mid, "v"), Some("123".to_string()));

        let url_none = "https://example.com";
        assert_eq!(extract_param(url_none, "v"), None);
    }

    #[test]
    fn test_get_title_from_json() {
        let json = r#"{
            "items": [
                {
                    "snippet": {
                        "title": "Test Title"
                    }
                }
            ]
        }"#;
        assert_eq!(get_title_from_json(json), Some("Test Title".to_string()));

        let empty_items = r#"{ "items": [] }"#;
        assert_eq!(get_title_from_json(empty_items), None);

        let bad_json = "{";
        assert_eq!(get_title_from_json(bad_json), None);
    }
}
