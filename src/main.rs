use std::{fs};
use actix_web::{ web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use dotenvy::dotenv;
use crate::download::send_download;

mod download;
mod db_link;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Recipient, StreamHandler};
use actix_web_actors::ws;

// --- DÉFINITION DE L'ACTEUR LOBBY ---
// C'est lui qui va centraliser les messages pour les envoyer à tous les clients
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub struct PalemachineLobby {
    sessions: Vec<Recipient<WsMessage>>,
}

impl Actor for PalemachineLobby {
    type Context = Context<Self>;
}

impl Handler<WsMessage> for PalemachineLobby {
    type Result = ();
    fn handle(&mut self, msg: WsMessage, _: &mut Context<Self>) {
        for session in &self.sessions {
            let _ = session.do_send(msg.clone());
        }
    }
}

// Message pour qu'une session s'enregistre auprès du lobby
#[derive(Message)]
#[rtype(result = "()")]
struct Connect { pub addr: Recipient<WsMessage> }

impl Handler<Connect> for PalemachineLobby {
    type Result = ();
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.push(msg.addr);
    }
}

// --- L'ACTEUR DE SESSION WEBSOCKET ---
struct MyWs {
    lobby_addr: Addr<PalemachineLobby>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // C'est ctx.address() et non self.address()
        let addr = ctx.address().recipient();
        self.lobby_addr.do_send(Connect { addr });
    }
}

impl Handler<WsMessage> for MyWs {
    type Result = ();
    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, _msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {}
}

// --- LA ROUTE WS ---
async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    lobby: web::Data<Addr<PalemachineLobby>>
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(MyWs { lobby_addr: lobby.get_ref().clone() }, &req, stream)
}

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
    album : String,
    artist : String,
    name: String,
    url: String,
    image: String,
       
}

async fn download(
    req: HttpRequest,
    query: web::Query<Downladstruct>,
    lobby: web::Data<Addr<PalemachineLobby>> // 1. On injecte le lobby ici
) -> impl Responder {
    if let Some(peer_addr) = req.peer_addr() {
        println!("Client IP: {}", peer_addr.ip());
        println!("📥 Route /downlad appelée pour : {}", &query.name);

        // 2. On clone les données pour pouvoir les déplacer (move) dans le thread async
        let url = query.url.clone();
        let name = query.name.clone();
        let image = query.image.clone();
        let album = query.album.clone();
        let artist = query.artist.clone();
        let lobby_addr = lobby.get_ref().clone();

        // 3. On lance le téléchargement en arrière-plan
        // Cela évite que la requête HTTP ne reste bloquée (timeout)
        tokio::spawn(async move {
            send_download(&url, &name, &image, &album, &artist, lobby_addr).await;
        });

        // 4. On répond immédiatement au client
        return HttpResponse::Ok()
            .body("Téléchargement lancé ! Surveillez les logs WebSocket.");
    }

    // Gestion d'erreur simplifiée
    HttpResponse::InternalServerError().body("Erreur de récupération d'IP")
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

async fn get_version() -> Result<String, reqwest::Error> {
    let githubversion = reqwest::get("https://raw.githubusercontent.com/Xmoncoco/palemachine/refs/heads/master/.version").await?;
    let version = githubversion.text().await?;
    Ok(version)
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
    let path = config
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            panic!("Champ 'path' manquant ou mal formé dans config.toml")
    });
    let lobby = PalemachineLobby { sessions: Vec::new() }.start();
    let lobby_data = web::Data::new(lobby);
    println!("path: {}", path);
    println!("the server has started at http://127.0.0.1:{}",port);    
    HttpServer::new(move ||
        App::new()
            .app_data(lobby_data.clone())
            .route("/", web::get().to(root))
            .route("/imagequestion", web::get().to(image_question))
            .route("/downlad",web::get().to(download))
            .route("/ws", web::get().to(ws_index))
    )
    .bind(("0.0.0.0",port))?
    .run()
    .await
}
