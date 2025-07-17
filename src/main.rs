use std::fs;
use actix_web::{web,App,HttpResponse,HttpServer,Responder,HttpRequest};
use serde::Deserialize;
mod download;

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
    friendly_name: String,
}

async fn image_question(req : HttpRequest, query : web::Query<ImageQuestion>) -> impl Responder{
    let yt_url = &query.yt_url;
    let friendly_name = &query.friendly_name;
    download::get_image(yt_url, friendly_name);
    if let Some(peer_addr) = req.peer_addr() {
        println!("Client IP: {}", peer_addr.ip());
    }
    let html = fs::read_to_string("pages/imageQuestion.html").unwrap_or_else(|_| {
        "<h1>Failed to read imageQuestion.html restart your server</h1>".to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    println!("{:?}", std::env::current_dir());
    println!("the server has started at 127.0.0.1:8080");

    HttpServer::new(||(
        App::new()
            .route("/", web::get().to(root))
            .route("/imagequestion", web::get().to(image_question))
    ))
    .bind(("127.0.0.1",8080))?
    .run()
    .await
}
