use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::EnvFilter;
use sqlx::MySqlPool;

mod models;
mod handlers;
mod db_mysql;

#[tokio::main]
async fn main() {
    // ✅ Logging activado (usa RUST_LOG=debug o info)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    println!("🧪 Iniciando backend...");

    // 🔌 Conexión MySQL
    let db_pool = db_mysql::init_pool().await;

    // 🛡️ CORS para desarrollo
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 📦 API con todas las rutas
    let api = Router::new()
        .route("/jugada", post(handlers::post_jugada))
        .route("/estado/:id_partida", get(handlers::get_estado))
        .route("/usuarios", get(handlers::get_usuarios))
        .route("/estadisticas/:id_usuario", get(handlers::get_estadisticas))
        .route("/formacion", post(handlers::post_formacion))
        .route("/registro", post(handlers::post_registro))  // Ruta nueva
        .route("/partida", post(handlers::post_partida))    // Ruta nueva
        .layer(Extension(db_pool));

    println!("🧪 Rutas API cargadas.");

    // ✅ Ruta absoluta al frontend (estático)
    let static_dir: PathBuf = PathBuf::from("../frontend");

    // 🔀 App principal combinada
    let app = Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .layer(cors);

    // 🚀 Servidor HTTP
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("✅ Servidor escuchando en http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}