use axum::{
    extract::Extension,
    routing::{get, post, get_service},   // ✅  sin `route_service`
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::EnvFilter;

mod models;
mod handlers;
mod db_mysql;

#[tokio::main]
async fn main() {
    /* ────────── logging ────────── */
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    /* ────────── DB pool ────────── */
    let db_pool = db_mysql::init_pool().await;

    /* ────────── API JSON ────────── */
    let api = Router::new()
        .route("/jugada",             post(handlers::post_jugada))
        .route("/estado/:id_partida", get(handlers::get_estado))
        .route("/usuarios",           get(handlers::get_usuarios))
        .route("/estadisticas/:id_usuario", get(handlers::get_estadisticas))
        .route("/formacion",          post(handlers::post_formacion))
        .route("/registro",           post(handlers::post_registro))
        .route("/partida",            post(handlers::post_partida))
        .layer(Extension(db_pool));

    /* ────────── archivos estáticos ────────── */
    // .../rustball_workspace/frontend
    let static_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend");
    let registro_file       = static_dir.join("registro.html");

    let static_root = Router::new()
        // /  →  registro.html
        .route("/", get_service(ServeFile::new(registro_file)))
        // /css/…, /js/…, /lobby.html, etc.
        .route_service("/*path", get_service(ServeDir::new(static_dir)));

    /* ────────── app final ────────── */
    let app = Router::new()
        .nest("/api", api)         // JSON bajo /api
        .merge(static_root)        // todo lo público
        .layer(
            CorsLayer::new()       // CORS amplio para dev
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    /* ────────── servidor ────────── */
    let addr     = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("✅ Servidor escuchando en http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
