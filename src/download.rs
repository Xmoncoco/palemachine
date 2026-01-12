use std::{ fs::{self, File}, process::Command};
use actix_web::http::header;
use reqwest::Client;
use chrono::Utc;
use crate::{ db_link::{self, add_entry} };
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

        let is_playlist = is_youtube_playlist(&url);

        if let Ok(youtube_api_key) = std::env::var("YOUTUBE_API_KEY") {
            if is_playlist {
                if let Some(id) = extract_param(&url, "list") {
                    println!("a new image ask with the playlist ID: {}", id);
                    return process_playlist(&id, &youtube_api_key, &url, &name, &ip).await;
                }
            } else {
                if let Some(id) = extract_param(&url, "v") {
                    println!("a new image ask with the youtube ID: {}", id);
                    return process_single_video(&id, &youtube_api_key, &url, &name, &ip).await;
                }
            }
        } else {
            eprintln!("have you set the YOUTUBE_API_KEY env variable?");
        }

        println!("No ID found in the URL");
        Vec::<SimpleSpotifyThumbnail>::new()
    }).await.unwrap()
}

fn is_youtube_playlist(url: &str) -> bool {
    url.contains("youtube.com/playlist") ||
    url.contains("youtu.be/playlist") ||
    (url.contains("list=") && !url.contains("&v="))
}

async fn process_single_video(
    id: &str,
    api_key: &str,
    url: &str,
    name: &str,
    ip: &str,
) -> Vec<SimpleSpotifyThumbnail> {
    let query = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=snippet&id={}&key={}",
        id, api_key
    );

    if let Some(body) = http_get(&query).await {
        if let Some(title) = get_title_from_json(&body) {
            let entry = db_link::DbEntry {
                url: url.to_string(),
                yt_id: id.to_string(),
                friendly_name: name.to_string(),
                real_name: title.clone(),
                timestamp: Utc::now().to_rfc3339(),
                ip: ip.to_string(),
            };
            let _ = add_entry(entry);

            let token = get_spotify_token().await;
            return get_thumbnails(&token, &title, name).await;
        }
    } else {
        eprintln!("Failed to fetch video details from YouTube API");
    }
    Vec::new()
}

async fn process_playlist(
    playlist_id: &str,
    api_key: &str,
    url: &str,
    name: &str,
    ip: &str,
) -> Vec<SimpleSpotifyThumbnail> {
    let query = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults=50&playlistId={}&key={}",
        playlist_id, api_key
    );

    if let Some(body) = http_get(&query).await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            let entry = db_link::DbEntry {
                url: url.to_string(),
                yt_id: playlist_id.to_string(),
                friendly_name: name.to_string(),
                real_name: format!("Playlist: {}", name),
                timestamp: Utc::now().to_rfc3339(),
                ip: ip.to_string(),
            };
            let _ = add_entry(entry);

            // For playlists, return first valid thumbnail
            if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                if let Some(first_item) = items.first() {
                    if let Some(title) = first_item
                        .get("snippet")
                        .and_then(|s| s.get("title"))
                        .and_then(|t| t.as_str())
                    {
                        let token = get_spotify_token().await;
                        return get_thumbnails(&token, title, name).await;
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to fetch playlist details from YouTube API");
    }
    Vec::new()
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

pub async fn send_download(url: &str, name: &str, image: &str, album: &str, artist: &str) {
    let url = url.to_string();
    let name = name.to_string();
    let image = image.to_string();
    let artist = artist.to_string();
    let album = album.to_string();

    let name_clone = name.clone();
    let is_playlist = is_youtube_playlist(&url);

    println!("🚀 Début du téléchargement...");

    let download_result = tokio::task::spawn_blocking(move || -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        println!("⏳ spawn_blocking démarré");

        let configfile = fs::read_to_string("config.toml")?;
        let config: toml::Value = toml::from_str(&configfile)?;
        let path = config
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("❌ Champ 'path' manquant dans config.toml")?
            .to_string();

        let python_bin = "./venv/bin/python3";
        let script_path = "downloader";

        if is_playlist {
            println!("📂 Mode Playlist détecté");
            println!("🐍 Lancement: {} {} playlist {} {} {}", python_bin, script_path, &url, &path, &name);

            let status = Command::new(python_bin)
                .arg(script_path)
                .arg("playlist")
                .arg(&url)
                .arg(&path)
                .arg(&name)
                .status()?;

            if status.success() {
                println!("✅ Playlist traitée par Python.");
                Ok(path)
            } else {
                let err_msg = format!("❌ Le script Python a échoué (code: {:?})", status.code());
                eprintln!("{}", err_msg);
                Err(err_msg.into())
            }
        } else {
            println!("🎵 Mode Musique détecté");
            println!("🐍 Lancement: {} {} single {} {} {} {} {}", python_bin, script_path, &url, &path, &name, &artist, &album);

            let status = Command::new(python_bin)
                .arg(script_path)
                .arg("single")
                .arg(&url)
                .arg(&path)
                .arg(&name)
                .arg(&artist)
                .arg(&album)
                .status()?;

            if status.success() {
                println!("✅ Musique traitée par Python.");
                Ok(path)
            } else {
                let err_msg = format!("❌ Le script Python a échoué (code: {:?})", status.code());
                eprintln!("{}", err_msg);
                Err(err_msg.into())
            }
        }
    })
    .await
    .expect("Erreur critique dans spawn_blocking");

    println!("📦 spawn_blocking terminé");

    match download_result {
        Ok(path) => {
            if !is_playlist {
                println!("🖼️ Téléchargement de la cover...");
                match download_image(&image, &path, &name_clone).await {
                    Ok(_) => println!("✅ Cover téléchargée avec succès."),
                    Err(e) => eprintln!("❌ Erreur téléchargement cover: {}", e),
                }
            } else {
                println!("ℹ️ Pas de cover à télécharger pour une playlist.");
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur: {}", e);
        }
    }
}

async fn download_image(url: &str,output_path: &str,name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let real_path = format!("{}/{}.jpg", output_path, name);

    let bytes = reqwest::get(url).await?.bytes().await?;

    tokio::fs::write(&real_path, &bytes).await?;

    Command::new("./bambam_morigatsu_chuapo.sh")
        .arg(output_path)
        .status()?;

    Ok(())
}
