use axum::{
    extract::Extension,
    routing::{get, post, get_service},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    cors::{CorsLayer, Any},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, error, Level};
use tracing_subscriber::EnvFilter;
use http::HeaderValue;

/* ────── Módulos ───────────────────────────────────────────── */
mod models;
mod handlers;
mod db_mysql;
mod routes;

use handlers::*;
use routes::websocket::websocket_handler;

#[tokio::main]
async fn main() {
    /* ─── Tracing / Logging ───────────────────────────────────── */
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    info!("🚀 Iniciando backend RustBall...");

    /* ─── Conexión a MySQL ───────────────────────────────────── */
    let db_pool = match db_mysql::init_pool().await {
        pool => {
            info!("✅ Conexión a MySQL establecida.");
            pool
        }
    };

    /* ─── API (REST + WebSocket) ─────────────────────────────── */
    let api = Router::new()
        .route("/jugada",               post(post_jugada))
        .route("/estado/:id",           get(get_estado))
        .route("/usuarios",             get(get_usuarios))
        .route("/estadisticas/:u",      get(get_estadisticas))
        .route("/formacion",            post(post_formacion))
        .route("/registro",             post(post_registro))
        .route("/login",                post(post_login))
        .route("/partida",              post(post_partida))
        .route("/mis_partidas/:u",      get(get_mis_partidas))
        .route("/gol",                  post(post_gol))
        .route("/snapshot/:p",          get(get_snapshot))
        .route("/pendientes/:u",        get(get_partidas_pendientes))
        .route("/partida_detalle/:p",   get(get_partida_detalle))
        .route("/ws/:partida/:uid",     get(websocket_handler))
        .layer(Extension(db_pool));

    /* ─── Archivos estáticos (SPA) ───────────────────────────── */
    let static_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("webapp");
    let landing = static_dir.join("registro.html");

    let static_site = Router::new()
        .route("/", get_service(ServeFile::new(landing.clone())))
        .route_service("/*path", get_service(ServeDir::new(static_dir.clone())));

    info!("📁 Archivos estáticos servidos desde: {:?}", static_dir);

    /* ─── Middlewares CORS + Trace ───────────────────────────── */
    let environment = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".into());

    let cors = if environment == "production" {
        info!("🌍 CORS restringido a: https://rustball.lat");
        CorsLayer::new()
            .allow_origin("https://rustball.lat".parse::<HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        info!("🌎 CORS en modo abierto (desarrollo)");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let app = Router::new()
        .nest("/api", api)
        .merge(static_site)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    /* ─── Arrancar servidor ───────────────────────────────────── */
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| {
            info!("🌐 No se encontró PORT, usando 10000 por defecto");
            "10000".into()
        })
        .parse()
        .unwrap_or_else(|e| {
            error!("❌ El PORT no es válido: {}", e);
            std::process::exit(1);
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🟢 Servidor escuchando en: http://{}", addr);

    if let Err(e) = axum::serve(TcpListener::bind(addr).await.unwrap(), app).await {
        error!("❌ Error al iniciar el servidor: {}", e);
    }
}


