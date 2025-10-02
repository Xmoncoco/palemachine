use std::fs;
use actix_web::{ web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use dotenvy::dotenv;
use crate::download::send_download;

mod download;
mod db_link;

async fn root(req : HttpRequest) -> impl Responder{
    if let Some(peer_addr) = req.peer_addr() {
        println!("Client IP: {}", peer_addr.ip());
    }
    let html = fs::read_to_string("pages/root.html").unwrap_or_else(|_| {
        "<h1>Failed to read index.html restart your server</h1>".to_string()
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
            .json(result); // ⬅️ vec automatiquement converti en JSON
    }

    let html = fs::read_to_string("pages/imageQuestion.html").unwrap_or_else(|_| {
        "<h1>Failed to read imageQuestion.html restart your server</h1>".to_string()
    });
    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    dotenv().ok();

    println!("{:?}", std::env::current_dir());
    println!("the server has started at 127.0.0.1:8080");
    
    let _db = db_link::init();
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
    println!("the server has started at 127.0.0.1:{}",port);
    
    HttpServer::new(||(
        App::new()
            .route("/", web::get().to(root))
            .route("/imagequestion", web::get().to(image_question))
            .route("/downlad",web::get().to(download))
    ))
    .bind(("127.0.0.1",port))?
    .run()
    .await
}
