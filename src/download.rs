use std::{ fs::{self}, process::Command};
use actix_web::http::header;
use reqwest::Client;
use chrono::Utc;
use crate::{ db_link::{self, add_entry} };
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use tokio::process::Command as TokioCommand; // Pour le script de permission
use std::process::Command as StdCommand; // Pour le script Python (dans spawn_blocking)
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


pub fn sanitise_name(name: &str) -> String {
    name.replace('/', "_")
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
                if let Some(id) = extract_param(&url, "list")  {
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
    if let Some(radio) = extract_param(url, "stratradio") {
        return radio != "1";
    }
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

    println!("🚀 Début du trigger Python...");

    // On prépare les données pour le thread bloquant
    let name_for_python = name.clone();

    // Tâche lourde (Python) -> Thread séparé
    let download_result = tokio::task::spawn_blocking(move || -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let configfile = fs::read_to_string("config.toml")?;
        let config: toml::Value = toml::from_str(&configfile)?;
        let base_path = config.get("path")
            .and_then(|v| v.as_str())
            .ok_or("❌ Config path manquant")?
            .to_string();

        let python_bin = "./venv/bin/python3";
        let script_path = "downloader"; // Ajoute l'extension .py si nécessaire
        let sanitised_name = sanitise_name(&name_for_python);

        // Construction de la commande (identique à ton code)
        let mut cmd = StdCommand::new(python_bin);
        cmd.arg(script_path);

        if is_youtube_playlist(&url) {
            cmd.args(&["playlist", &url, &base_path, &sanitised_name]);
        } else {
            cmd.args(&["single", &url, &base_path, &sanitised_name, &artist, &album]);
        }

        println!("🐍 Exécution Python...");
        let output = cmd.output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // --- CORRECTION CRITIQUE ARIA2C ---
            // On ne prend pas aveuglément la dernière ligne.
            // On cherche la dernière ligne qui est un chemin valide (commence par / ou contient le base_path)
            let valid_path = stdout.lines()
                .rev() // On part de la fin
                .find(|line| line.trim().starts_with("/") || line.contains(&base_path)) // Critère de filtrage
                .map(|l| l.trim().to_string());

            if let Some(path) = valid_path {
                println!("✅ Python Success. Chemin capturé: {}", path);
                Ok(path)
            } else {
                // Fallback: Si Python n'a rien craché de propre, on devine le chemin
                // C'est risqué mais mieux que de crash
                eprintln!("⚠️ Attention: Python n'a pas retourné de chemin clair via stdout (bruit aria2c?). Utilisation du chemin par défaut.");
                if is_youtube_playlist(&url) {
                    Ok(format!("{}/Playlists/{}", base_path, sanitised_name))
                } else {
                    Ok(format!("{}/{}/{}", base_path, sanitise_name(&artist), sanitise_name(&album)))
                }
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("❌ Python Error (Code {:?}): {}", output.status.code(), stderr).into())
        }
    }).await;

    // Gestion du résultat du thread
    match download_result {
        Ok(Ok(returned_path)) => {
            println!("🖼️ Récupération de la cover vers : {}", returned_path);
            if let Err(e) = download_image(&image, &returned_path, &name).await {
                eprintln!("❌ Erreur download_image: {}", e);
            }
        },
        Ok(Err(e)) => eprintln!("{}", e), // Erreur du script Python
        Err(e) => eprintln!("❌ Erreur JoinHandle: {}", e), // Crash du thread
    }
}

async fn download_image(url: &str, output_path: &String, name: &String) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Création dossier (Async I/O)
    tokio::fs::create_dir_all(output_path).await?;

    let clean_name = sanitise_name(name);
    let file_path = format!("{}/{}.jpg", output_path, clean_name);

    // 2. Téléchargement (Async HTTP)
    let bytes = reqwest::get(url).await?.bytes().await?;
    tokio::fs::write(&file_path, &bytes).await?;
    println!("📸 Cover sauvegardée: {}", file_path);

    // 3. Script de permissions (CORRECTION ASYNC)
    // On utilise tokio::process::Command au lieu de std::process::Command
    // pour ne pas bloquer le serveur web pendant l'exécution du script bash
    println!("🔧 Lancement du script de permissions...");
    let status = TokioCommand::new("./bambam_morigatsu_chuapo")
        .arg(output_path)
        .kill_on_drop(true) // Sécurité si la requête est annulée
        .status()
        .await?;

    if !status.success() {
        eprintln!("❌ Le script bambam a échoué avec le code: {:?}", status.code());
    } else {
        println!("✅ Permissions appliquées.");
    }

    Ok(())
}