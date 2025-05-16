use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use crate::models::*;

// 1. POST /jugada
#[axum::debug_handler]
pub async fn post_jugada(
    Extension(pool): Extension<MySqlPool>,
    Json(payload): Json<JugadaPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let result = sqlx::query!(
        "INSERT INTO Turno (id_partida, numero_turno, id_usuario, jugada)
         VALUES (?, ?, ?, ?)",
        payload.id_partida,
        payload.numero_turno,
        payload.id_usuario,
        payload.jugada
    )
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(Json("Turno registrado")),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 2. GET /estado/:id_partida
#[axum::debug_handler]
pub async fn get_estado(
    Path(id_partida): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Vec<TurnoData>>, (StatusCode, String)> {
    let turnos = sqlx::query_as!(
        TurnoData,
        r#"
        SELECT
            numero_turno,
            id_usuario,
            jugada,
            fecha_turno AS "fecha_turno: chrono::NaiveDateTime"
        FROM Turno
        WHERE id_partida = ?
        ORDER BY numero_turno ASC
        "#,
        id_partida
    )
        .fetch_all(&pool)
        .await;

    match turnos {
        Ok(t) => Ok(Json(t)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 3. GET /usuarios
#[axum::debug_handler]
pub async fn get_usuarios(
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Vec<Usuario>>, (StatusCode, String)> {
    println!("🧪 Entrando al handler GET /usuarios...");

    let result = sqlx::query!(
        r#"
        SELECT
            id_usuario,
            nombre_usuario,
            correo,
            contrasena
        FROM Usuario
        "#
    )
        .fetch_all(&pool)
        .await;

    match result {
        Ok(rows) => {
            println!("✅ Filas recibidas: {}", rows.len());

            let usuarios: Vec<Usuario> = rows.into_iter().map(|row| Usuario {
                id_usuario: row.id_usuario,
                nombre_usuario: row.nombre_usuario,
                correo: row.correo,
                contrasena: row.contrasena,
            }).collect();

            Ok(Json(usuarios))
        }
        Err(e) => {
            println!("❌ Error en SQLx: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

// 4. GET /estadisticas/:id_usuario
#[axum::debug_handler]
pub async fn get_estadisticas(
    Path(id_usuario): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Estadistica>, (StatusCode, String)> {
    let estad = sqlx::query_as!(
        Estadistica,
        "SELECT id_usuario, partidas_jugadas, partidas_ganadas, goles_a_favor, goles_en_contra
         FROM Estadistica
         WHERE id_usuario = ?",
        id_usuario
    )
        .fetch_optional(&pool)
        .await;

    match estad {
        Ok(Some(e)) => Ok(Json(e)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "No se encontraron estadísticas para este usuario".into())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 5. POST /formacion
#[axum::debug_handler]
pub async fn post_formacion(
    Extension(pool): Extension<MySqlPool>,
    Json(payload): Json<FormacionPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let result = sqlx::query!(
        "INSERT INTO FormacionElegida (id_partida, id_usuario, formacion, turno_inicio)
         VALUES (?, ?, ?, ?)",
        payload.id_partida,
        payload.id_usuario,
        payload.formacion,
        payload.turno_inicio
    )
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(Json("Formación registrada")),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 6. POST /registro
use serde_json::json;

#[axum::debug_handler]
pub async fn post_registro(
    Extension(pool): Extension<MySqlPool>,
    Json(payload): Json<RegistroPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let res = sqlx::query!(
        "INSERT INTO Usuario (nombre_usuario, correo, contrasena) VALUES (?, ?, ?)",
        payload.nombre_usuario,
        payload.correo,
        payload.contrasena
    )
        .execute(&pool)
        .await;

    match res {
        Ok(r) => {
            let id = r.last_insert_id() as i32;
            Ok(Json(json!({
                "id_usuario": id,
                "nombre_usuario": payload.nombre_usuario,
                "correo": payload.correo
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 7. POST /partida - CORREGIDO para MySQL
#[axum::debug_handler]
pub async fn post_partida(
    Extension(pool): Extension<MySqlPool>,
    Json(payload): Json<PartidaPayload>,
) -> Result<Json<Partida>, (StatusCode, String)> {
    // Verificar si ya existe la partida
    let existente = sqlx::query!(
        r#"
        SELECT
            id_partida,
            id_jugador1,
            id_jugador2,
            fecha_inicio AS "fecha_inicio: chrono::NaiveDateTime"
        FROM Partida
        WHERE (id_jugador1 = ? AND id_jugador2 = ?)
           OR (id_jugador1 = ? AND id_jugador2 = ?)
        "#,
        payload.id_usuario_1,
        payload.id_usuario_2,
        payload.id_usuario_2,
        payload.id_usuario_1
    )
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(row) = existente {
        let partida = Partida {
            id_partida: row.id_partida,
            id_usuario_1: row.id_jugador1,
            id_usuario_2: row.id_jugador2,
            fecha_creacion: row.fecha_inicio,
        };
        return Ok(Json(partida));
    }

    // Crear nueva partida (en MySQL, necesitamos dos consultas separadas)
    let result = sqlx::query!(
        r#"
        INSERT INTO Partida (id_jugador1, id_jugador2)
        VALUES (?, ?)
        "#,
        payload.id_usuario_1,
        payload.id_usuario_2
    )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Obtener el ID de la última inserción y los datos de la partida
    let partida_id = result.last_insert_id() as i32;

    let nueva_row = sqlx::query!(
        r#"
        SELECT
            id_partida,
            id_jugador1,
            id_jugador2,
            fecha_inicio AS "fecha_inicio: chrono::NaiveDateTime"
        FROM Partida
        WHERE id_partida = ?
        "#,
        partida_id
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let partida = Partida {
        id_partida: nueva_row.id_partida,
        id_usuario_1: nueva_row.id_jugador1,
        id_usuario_2: nueva_row.id_jugador2,
        fecha_creacion: nueva_row.fecha_inicio,
    };

    Ok(Json(partida))
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub nombre_usuario: String,
    pub contrasena: String,
}

#[axum::debug_handler]
pub async fn post_login(
    Extension(pool): Extension<MySqlPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Usuario>, (StatusCode, String)> {
    let resultado = sqlx::query_as!(
        Usuario,
        "SELECT id_usuario, nombre_usuario, correo, contrasena
         FROM Usuario
         WHERE nombre_usuario = ? AND contrasena = ?",
        payload.nombre_usuario,
        payload.contrasena
    )
        .fetch_optional(&pool)
        .await;

    match resultado {
        Ok(Some(usuario)) => Ok(Json(usuario)),
        Ok(None) => Err((StatusCode::UNAUTHORIZED, "Credenciales inválidas".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
