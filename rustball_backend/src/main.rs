//! src/main.rs
//! ------------------------------------------------------------------
//! • Sirve la API REST en  /api/*
//! • Sirve los archivos de la SPA (webapp/) en la raíz  /
//! • Expone un WebSocket en  /api/ws/{partida}/{uid}
//! • Habilita CORS *  +  trace de peticiones en desarrollo
//! ------------------------------------------------------------------

use axum::{
    extract::Extension,
    routing::{get, post, get_service},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

/* ────── Módulos de la crate ─────────────────────────────────────── */
mod models;
mod handlers;
mod db_mysql;
mod routes; // ↳ routes::websocket

use handlers::*;
use routes::websocket::websocket_handler;

#[tokio::main]
async fn main() {
    /* ─── Tracing / logging ──────────────────────────────────────── */
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)              //  ◀ DEBUG global
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    /* ─── Pool MySQL ─────────────────────────────────────────────── */
    let db_pool = db_mysql::init_pool().await;

    /* ─── Canal broadcast WebSocket ──────────────────────────────── */
    let (tx, _rx) = broadcast::channel::<String>(100);

    /* ─── Router REST + WS (/api) ────────────────────────────────── */
    let api = Router::new()
        /* REST ---------------------------------------------------- */
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
        /* WebSocket ---------------------------------------------- */
        .route("/ws/:partida/:uid",     get(websocket_handler))
        /* Estado compartido -------------------------------------- */
        .layer(Extension(db_pool))
        .layer(Extension(tx.clone()));

    /* ─── Archivos estáticos (SPA) ──────────────────────────────── */
    let static_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("webapp");
    let landing = static_dir.join("registro.html");

    let static_site = Router::new()
        .route("/", get_service(ServeFile::new(landing)))
        .route_service("/*path", get_service(ServeDir::new(static_dir)));

    /* ─── CORS y trace (solo dev) ───────────────────────────────── */
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    /* ─── Router raíz ───────────────────────────────────────────── */
    let app = Router::new()
        .nest("/api", api)
        .merge(static_site)
        .layer(cors)                       //  ◀ primero CORS
        .layer(TraceLayer::new_for_http()); // ◀ luego Trace

    /* ─── Arrancar servidor ─────────────────────────────────────── */
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "10000".into())
        .parse()
        .expect("PORT debe ser numérico");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("✅ Backend escuchando en http://{addr}");

    axum::serve(TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
