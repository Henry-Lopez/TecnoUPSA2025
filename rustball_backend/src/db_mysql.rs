use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use dotenvy::dotenv;
use std::env;

/// Inicializa el pool de conexión a MySQL
pub async fn init_pool() -> MySqlPool {
    // Carga las variables desde .env
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("⚠️ DATABASE_URL no definida en .env");

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("❌ No se pudo conectar a la base de datos")
}