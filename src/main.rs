use std::{fs, process::Command};
use actix_web::{ web, App, HttpRequest, HttpResponse, HttpServer, Responder, cookie::Cookie};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use dotenvy::dotenv;
use crate::download::send_download;

mod download;
mod db_link;

struct AppState {
    password: String,
    webauthn: Arc<Webauthn>,
    // Map session_id -> (reg_state, login_state)
    // In a real app, use a proper session store (Redis, DB)
    auth_states: Arc<Mutex<HashMap<Uuid, AuthState>>>,
}

enum AuthState {
    Register(PasskeyRegistration),
    Login(PasskeyAuthentication),
}

#[derive(Deserialize)]
struct LoginData {
    passkey: String,
}

// --- WebAuthn Endpoints ---

async fn register_start(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    // Only allow registration if already logged in via password (cookie check)
    if let Some(cookie) = req.cookie("access_token") {
        if cookie.value() != "granted" {
            return HttpResponse::Unauthorized().body("Must be logged in to register passkey");
        }
    } else {
        return HttpResponse::Unauthorized().body("Must be logged in to register passkey");
    }

    let user_unique_id = Uuid::nil(); // Single user "admin"
    let passkeys = db_link::get_passkeys(user_unique_id).unwrap_or_default();

    let exclude_credentials = Some(passkeys.iter().map(|p| p.cred_id().clone()).collect());

    match state.webauthn.start_passkey_registration(user_unique_id, "Admin", "Admin", exclude_credentials) {
        Ok((ccr, reg_state)) => {
            let session_id = Uuid::new_v4();
            state.auth_states.lock().unwrap().insert(session_id, AuthState::Register(reg_state));
            
            // Send session_id in cookie or response. Here we send in response for simplicity of the JS client
            HttpResponse::Ok()
                .cookie(Cookie::build("auth_session", session_id.to_string()).path("/").finish())
                .json(ccr)
        }
        Err(e) => {
            eprintln!("Register start error: {:?}", e);
            HttpResponse::InternalServerError().body("Error starting registration")
        }
    }
}

async fn register_finish(state: web::Data<AppState>, req: HttpRequest, json: web::Json<RegisterPublicKeyCredential>) -> impl Responder {
    let session_id = req.cookie("auth_session")
        .and_then(|c| Uuid::parse_str(c.value()).ok())
        .unwrap_or_default();

    let reg_state = {
        let mut states = state.auth_states.lock().unwrap();
        if let Some(AuthState::Register(s)) = states.remove(&session_id) {
            s
        } else {
            return HttpResponse::BadRequest().body("Invalid session");
        }
    };

    match state.webauthn.finish_passkey_registration(&json, &reg_state) {
        Ok(passkey) => {
            let user_unique_id = Uuid::nil();
            db_link::add_passkey(user_unique_id, &passkey).expect("Failed to save passkey");
            HttpResponse::Ok().body("Passkey registered!")
        }
        Err(e) => {
            eprintln!("Register finish error: {:?}", e);
            HttpResponse::BadRequest().body("Registration failed")
        }
    }
}

async fn login_start(state: web::Data<AppState>) -> impl Responder {
    let user_unique_id = Uuid::nil();
    let passkeys = db_link::get_passkeys(user_unique_id).unwrap_or_default();

    match state.webauthn.start_passkey_authentication(&passkeys) {
        Ok((rcr, auth_state)) => {
            let session_id = Uuid::new_v4();
            state.auth_states.lock().unwrap().insert(session_id, AuthState::Login(auth_state));
            
            HttpResponse::Ok()
                .cookie(Cookie::build("auth_session", session_id.to_string()).path("/").finish())
                .json(rcr)
        }
        Err(e) => {
             eprintln!("Login start error: {:?}", e);
             HttpResponse::InternalServerError().body("Error starting login")
        }
    }
}

async fn login_finish(state: web::Data<AppState>, req: HttpRequest, json: web::Json<PublicKeyCredential>) -> impl Responder {
    let session_id = req.cookie("auth_session")
        .and_then(|c| Uuid::parse_str(c.value()).ok())
        .unwrap_or_default();

    let auth_state = {
        let mut states = state.auth_states.lock().unwrap();
        if let Some(AuthState::Login(s)) = states.remove(&session_id) {
            s
        } else {
            return HttpResponse::BadRequest().body("Invalid session");
        }
    };

    match state.webauthn.finish_passkey_authentication(&json, &auth_state) {
        Ok(_) => {
            let cookie = Cookie::build("access_token", "granted")
                .path("/")
                .http_only(true)
                .finish();
            HttpResponse::Ok().cookie(cookie).body("Login successful")
        }
        Err(e) => {
            eprintln!("Login finish error: {:?}", e);
            HttpResponse::Unauthorized().body("Login failed")
        }
    }
}

async fn login(data: web::Json<LoginData>, state: web::Data<AppState>) -> impl Responder {
    if data.passkey == state.password {
        let cookie = Cookie::build("access_token", "granted")
            .path("/")
            .http_only(true)
            .finish();
            
        HttpResponse::Ok()
            .cookie(cookie)
            .body("Login successful")
    } else {
        HttpResponse::Unauthorized().body("Mot de passe incorrect")
    }
}

async fn root(req : HttpRequest) -> impl Responder{
    if let Some(cookie) = req.cookie("access_token") {
        if cookie.value() == "granted" {
            if let Some(peer_addr) = req.peer_addr() {
                println!("Client IP: {}", peer_addr.ip());
            }
            let html = fs::read_to_string("pages/root.html").unwrap_or_else(|_| {
                "<h1>Failed to read index.html restart your server</h1>".to_string()
            });
        
            return HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html);
        }
    }

    let html = fs::read_to_string("pages/login.html").unwrap_or_else(|_| {
        "<h1>Failed to read login.html</h1>".to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[derive(Deserialize)]
struct ImageQuestion {
    yt_url: String,
    friendlyname: String,
}

#[derive(Deserialize)]
struct Downladstruct {
    name: String,
    url: String,
    image: String,
       
}

async fn download(req: HttpRequest, query: web::Query<Downladstruct>) -> impl Responder {
    if let Some(peer_addr) = req.peer_addr() {
        println!("Client IP: {}", peer_addr.ip());
        
        send_download(&query.url, &query.name, &query.image).await;
        return HttpResponse::Ok()
            .body("hello world");
    }

    let html = fs::read_to_string("pages/imageQuestion.html").unwrap_or_else(|_| {
        "<h1>Failed to read imageQuestion.html restart your server</h1>".to_string()
    });
    HttpResponse::Ok().body(html)
}

async fn image_question(req: HttpRequest, query: web::Query<ImageQuestion>) -> impl Responder {
    if let Some(peer_addr) = req.peer_addr() {
        println!("Client IP: {}", peer_addr.ip());
        let ip = peer_addr.ip().to_string();

        let result: Vec<download::SimpleSpotifyThumbnail> =
            download::get_image(&query.yt_url, &query.friendlyname, &ip).await;

        return HttpResponse::Ok()
            .content_type("application/json")
            .json(result); 
    }

    let html = fs::read_to_string("pages/imageQuestion.html").unwrap_or_else(|_| {
        "<h1>Failed to read imageQuestion.html restart your server</h1>".to_string()
    });
    HttpResponse::Ok().body(html)
}

async fn get_version()-> Result<String, reqwest::Error>{
    let githubversion = reqwest::get("https://raw.githubusercontent.com/Xmoncoco/palemachine/refs/heads/master/.version").await?;
    let version = githubversion.text().await?;
    return Ok(version);
}

async fn update_system(req: HttpRequest) -> impl Responder {
    if let Some(peer_addr) = req.peer_addr() {
        println!("Update request from IP: {}", peer_addr.ip());
    }

    let result = tokio::task::spawn_blocking(|| {
        let script_path = if std::path::Path::new("./update.sh").exists() {
            "./update.sh"
        } else if std::path::Path::new("../update.sh").exists() {
            "../update.sh"
        } else {
            return Err("update.sh not found".to_string());
        };

        let output = Command::new(script_path).output().map_err(|e| e.to_string())?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }).await;

    match result {
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    path: String,
    port: u16,
    password: String,
    domain: String,
}

async fn get_config(req: HttpRequest) -> impl Responder {
    // Security check: must be logged in
    if let Some(cookie) = req.cookie("access_token") {
        if cookie.value() != "granted" {
             return HttpResponse::Unauthorized().body("Unauthorized");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let configfile = fs::read_to_string("config.toml").unwrap_or_default();
    // Parse toml to struct to ensure valid json response
    let config: Config = toml::from_str(&configfile).unwrap_or(Config {
        path: "".to_string(),
        port: 9999,
        password: "admin".to_string(),
        domain: "localhost".to_string(),
    });
    
    HttpResponse::Ok().json(config)
}

async fn save_config(req: HttpRequest, new_config: web::Json<Config>) -> impl Responder {
    // Security check: must be logged in
    if let Some(cookie) = req.cookie("access_token") {
        if cookie.value() != "granted" {
             return HttpResponse::Unauthorized().body("Unauthorized");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let toml_string = toml::to_string(&*new_config).expect("Failed to serialize config");
    
    match fs::write("config.toml", toml_string) {
        Ok(_) => HttpResponse::Ok().body("Config saved"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to write config: {}", e)),
    }
}

async fn settings_page(req: HttpRequest) -> impl Responder {
    // Security check
    if let Some(cookie) = req.cookie("access_token") {
        if cookie.value() != "granted" {
             return HttpResponse::Found().append_header(("Location", "/")).finish();
        }
    } else {
        return HttpResponse::Found().append_header(("Location", "/")).finish();
    }

    let html = fs::read_to_string("pages/settings.html").unwrap_or_else(|_| {
        "<h1>Failed to read settings.html</h1>".to_string()
    });
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{

    dotenv().ok();
    let version = fs::read_to_string(".version")
        .expect(".version");

    let local_version = version.trim();
    match get_version().await {
        Ok(github_version) => {
            let remote_version = github_version.trim();

            if remote_version == local_version {
                println!("✅ Latest version: {}", local_version);
            } else {
                println!("⚠️ New version available: {}", remote_version);
            }
        },
        Err(e) => {
            println!("❌ Unable to get the remote version: {}", e);
        }
    }
    let _db = db_link::init();
    
    if !std::path::Path::new("config.toml").exists() {
        println!("⚠️ config.toml not found. First time setup: Creating default configuration.");
        let default_config = r#"path = "./downloads"
port = 9999
password = "admin"
domain = "localhost"
"#;
        fs::write("config.toml", default_config).expect("Failed to create default config.toml");
    }

    let configfile = fs::read_to_string("config.toml")
            .expect("config.toml manquant !");
    let config: toml::Value = toml::from_str(&configfile)
            .expect("Erreur de parsing de config.toml");
    let port: u16 = config
        .get("port")
        .and_then(|v| v.as_integer()) // si t;u stockes un nombre dans le TOML
        .map(|v| v as u16)
        .unwrap_or_else(|| {
            panic!("Champ 'port' manquant ou mal formé dans config.toml")
    });
    let password = config.get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("admin")
        .to_string();
    
    let domain = config.get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("localhost");

    let rp_id = domain;
    let rp_origin = Url::parse(&format!("https://{}", domain)).unwrap_or_else(|_| Url::parse(&format!("http://{}", domain)).unwrap());
    
    let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid WebAuthn config");
    let webauthn = Arc::new(builder.build().expect("Failed to build WebAuthn"));
    let auth_states = Arc::new(Mutex::new(HashMap::new()));

    println!("the server has started at 127.0.0.1:{}",port);
    
    HttpServer::new(move ||
        App::new()
            .app_data(web::Data::new(AppState { 
                password: password.clone(),
                webauthn: webauthn.clone(),
                auth_states: auth_states.clone()
            }))
            .route("/", web::get().to(root))
            .route("/login", web::post().to(login))
            .route("/auth/register_start", web::post().to(register_start))
            .route("/auth/register_finish", web::post().to(register_finish))
            .route("/auth/login_start", web::post().to(login_start))
            .route("/auth/login_finish", web::post().to(login_finish))
            .route("/imagequestion", web::get().to(image_question))
            .route("/downlad",web::get().to(download))
            .route("/update", web::get().to(update_system))
            .route("/settings", web::get().to(settings_page))
            .route("/api/config", web::get().to(get_config))
            .route("/api/config", web::post().to(save_config))
    )
    .bind(("0.0.0.0",port))?
    .run()
    .await
}
