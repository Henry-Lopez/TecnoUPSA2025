use axum::{
    extract::Extension,
    routing::{get, post, get_service},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::EnvFilter;

use crate::handlers::get_partidas_pendientes;
use crate::routes::websocket::websocket_handler;

mod models;
mod handlers;
mod db_mysql;
mod routes;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_pool = db_mysql::init_pool().await;

    // Canal WebSocket (broadcast)
    let (tx, _rx) = broadcast::channel::<String>(100);

    let api = Router::new()
        .route("/jugada", post(handlers::post_jugada))
        .route("/estado/:id_partida", get(handlers::get_estado))
        .route("/usuarios", get(handlers::get_usuarios))
        .route("/estadisticas/:id_usuario", get(handlers::get_estadisticas))
        .route("/formacion", post(handlers::post_formacion))
        .route("/registro", post(handlers::post_registro))
        .route("/login", post(handlers::post_login))
        .route("/partida", post(handlers::post_partida))
        .route("/mis_partidas/:id_usuario", get(handlers::get_mis_partidas))
        .route("/gol", post(handlers::post_gol))
        .route("/snapshot/:id_partida", get(handlers::get_snapshot))
        .route("/pendientes/:id_usuario", get(get_partidas_pendientes))
        .route("/partida_detalle/:id_partida", get(handlers::get_partida_detalle))
        .route("/ws/:partida_id/:user_id", get(websocket_handler))
        .layer(Extension(db_pool))
        .layer(Extension(tx.clone())); // canal para WebSocket

    let static_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./webapp");
    let registro_file = static_dir.join("registro.html");

    let static_root = Router::new()
        .route("/", get_service(ServeFile::new(registro_file)))
        .route_service("/*path", get_service(ServeDir::new(static_dir)));

    let app = Router::new()
        .nest("/api", api)
        .merge(static_root)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "10000".to_string()) // Valor por defecto si PORT no está definido
    .parse::<u16>()
    .expect("PORT debe ser un número válido");

    let addr = SocketAddr::from(([0,0,0,0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("✅ Servidor escuchando en http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
