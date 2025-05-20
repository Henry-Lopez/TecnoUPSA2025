use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use crate::models::*;

#[axum::debug_handler]
pub async fn post_jugada(
    Extension(pool): Extension<MySqlPool>,
    Extension(tx): Extension<broadcast::Sender<String>>,
    Json(payload): Json<JugadaPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    // 1. Obtener el máximo número de turno actual
    let max_turno = sqlx::query_scalar!(
        "SELECT MAX(numero_turno) FROM Turno WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    let nuevo_turno = max_turno + 1;
    println!("🔢 Turno actual más alto: {}, nuevo_turno: {}", max_turno, nuevo_turno);

    // 2. Guardar el nuevo turno
    let result = sqlx::query!(
        "INSERT INTO Turno (id_partida, numero_turno, id_usuario, jugada)
         VALUES (?, ?, ?, ?)",
        payload.id_partida,
        nuevo_turno,
        payload.id_usuario,
        payload.jugada
    )
        .execute(&pool)
        .await;

    if let Err(e) = &result {
        let code = e.as_database_error().and_then(|d| d.code()).map(|s| s.to_string());
        println!("❌ Error al insertar turno: {:?}", e);
        if code.as_deref() == Some("1062") {
            return Err((StatusCode::CONFLICT, "Número de turno duplicado".into()));
        } else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    }

    println!("✅ Turno insertado correctamente");

    // 3. Obtener los jugadores de la partida
    let partida = sqlx::query!(
        "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Alternar turno: usamos valores lógicos 1 o 2
    let turno_logico_actual = sqlx::query_scalar!(
        "SELECT turno_actual FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .unwrap_or(Some(1))
        .unwrap_or(1);

    let nuevo_turno_logico = if turno_logico_actual == 1 { 2 } else { 1 };
    println!("🎯 Turno actual lógico: {}, próximo: {}", turno_logico_actual, nuevo_turno_logico);

    // 5. Actualizar turno_actual con el valor lógico
    sqlx::query!(
        "UPDATE Partida SET turno_actual = ? WHERE id_partida = ?",
        nuevo_turno_logico,
        payload.id_partida
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            println!("❌ Error al actualizar turno_actual: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    println!("🔄 turno_actual actualizado a: {}", nuevo_turno_logico);

    // 6. Notificar por WebSocket
    let _ = tx.send("turno_finalizado".to_string());
    println!("📡 Notificación enviada por WebSocket");

    Ok(Json("Turno registrado"))
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

// 5. POST /formacion ─────────────────────────────────────────────────────
#[axum::debug_handler]
pub async fn post_formacion(
    Extension(pool): Extension<MySqlPool>,
    Json(p): Json<FormacionPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    // 1. Guardar o actualizar la formación
    sqlx::query!(
        r#"
        INSERT INTO FormacionElegida (id_partida, id_usuario, formacion, turno_inicio)
        VALUES (?, ?, ?, 0)
        ON DUPLICATE KEY UPDATE
            formacion = VALUES(formacion)
        "#,
        p.id_partida,
        p.id_usuario,
        p.formacion
    )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Verificar si ya hay 2 formaciones
    let formaciones = sqlx::query!(
        "SELECT id_usuario FROM FormacionElegida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if formaciones.len() == 2 {
        let jugador1 = formaciones[0].id_usuario;
        let jugador2 = formaciones[1].id_usuario;

        // 3. Elegir al azar quién empieza
        let (primero, segundo) = if rand::random() {
            (jugador1, jugador2)
        } else {
            (jugador2, jugador1)
        };

        // 4. Actualizar los turno_inicio en la tabla
        sqlx::query!(
            "UPDATE FormacionElegida SET turno_inicio = 1 WHERE id_partida = ? AND id_usuario = ?",
            p.id_partida,
            primero
        )
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query!(
            "UPDATE FormacionElegida SET turno_inicio = 2 WHERE id_partida = ? AND id_usuario = ?",
            p.id_partida,
            segundo
        )
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // 5. Actualizar el estado de la partida y turno_actual
        sqlx::query!(
            r#"
            UPDATE Partida
            SET    estado = 'playing',
                   turno_actual = ?
            WHERE  id_partida = ?
            "#,
            primero,
            p.id_partida
        )
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // 6. Enviar snapshot actualizado inmediatamente
        //    (esto reemplaza lo que normalmente harías por WebSocket si fuera en tiempo real)
        let snapshot = get_snapshot(
            Path(p.id_partida),
            Extension(pool.clone())
        )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al generar snapshot: {:?}", e)))?
            .0;

        // (Opcional) Aquí podrías enviar el snapshot por WebSocket a los jugadores.
        // Por ahora, solo aseguramos que esté generado. Si el frontend usa polling, con esto basta.
        println!("📦 Snapshot generado y listo para ser leído por el frontend");
    }

    Ok(Json("Formación registrada"))
}


// 6. POST /registro
use serde_json::json;
use tokio::sync::broadcast;

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
            fecha_inicio AS "fecha_inicio: chrono::NaiveDateTime",
            estado
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
            fecha_creacion: row.fecha_inicio, // ✅ ya es Option<NaiveDateTime>
            estado: row.estado,
        };

        return Ok(Json(partida));
    }

    // Crear nueva partida (estado 'waiting' por defecto)
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

    let partida_id = result.last_insert_id() as i32;

    let nueva_row = sqlx::query!(
        r#"
        SELECT
            id_partida,
            id_jugador1,
            id_jugador2,
            fecha_inicio AS "fecha_inicio: chrono::NaiveDateTime",
            estado
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
        estado: nueva_row.estado,
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

#[axum::debug_handler]
pub async fn get_mis_partidas(
    Path(id_usuario): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Vec<Partida>>, (StatusCode, String)> {
    let partidas = sqlx::query_as!(
        Partida,
        r#"
        SELECT
            id_partida,
            id_jugador1 AS id_usuario_1,
            id_jugador2 AS id_usuario_2,
            fecha_inicio AS "fecha_creacion: chrono::NaiveDateTime",
            estado
        FROM Partida
        WHERE id_jugador1 = ? OR id_jugador2 = ?
        ORDER BY fecha_inicio DESC
        "#,
        id_usuario,
        id_usuario
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(partidas))
}

#[axum::debug_handler]
pub async fn post_gol(
    Extension(pool): Extension<MySqlPool>,
    Json(p): Json<GolPayload>,
) -> Result<Json<(i32, i32)>, (StatusCode, String)> {
    // Obtener quién es j1 y j2
    let row = sqlx::query!(
        "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Ejecutar el UPDATE correcto
    if p.id_goleador == row.id_jugador1 {
        sqlx::query!(
            "UPDATE Partida SET gol_j1 = gol_j1 + 1 WHERE id_partida = ?",
            p.id_partida
        )
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        sqlx::query!(
            "UPDATE Partida SET gol_j2 = gol_j2 + 1 WHERE id_partida = ?",
            p.id_partida
        )
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Consultar marcador actualizado
    let marcador = sqlx::query!(
        "SELECT gol_j1, gol_j2 FROM Partida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json((
        marcador.gol_j1.unwrap_or(0),
        marcador.gol_j2.unwrap_or(0),
    )))

}
// GET /snapshot/:id_partida ──────────────────────────────────────────────
#[axum::debug_handler]
pub async fn get_snapshot(
    Path(id_partida): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Snapshot>, (StatusCode, String)> {
    // ── 0) estado de la partida ─────────────────────────────────────────
    let mut partida = sqlx::query!(
        r#"
        SELECT
            estado        AS "estado!: String",
            turno_actual  -- Option<i32>
        FROM Partida
        WHERE id_partida = ?
        "#,
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    println!("🔎 Estado actual de la partida {id_partida}: {}, turno_actual: {:?}", partida.estado, partida.turno_actual);

    // ── 1) formaciones ──────────────────────────────────────────────────
    let formaciones = sqlx::query_as!(
        FormacionData,
        "SELECT id_usuario, formacion, turno_inicio
         FROM   FormacionElegida
         WHERE  id_partida = ?",
        id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ── 2) Si hay menos de 2 formaciones devolvemos snapshot incompleto ─
    if formaciones.len() < 2 {
        println!("⚠️ Faltan formaciones. Solo hay {}", formaciones.len());
        return Ok(Json(Snapshot {
            marcador: (0, 0),
            formaciones,
            turnos: vec![],
            proximo_turno: 0,
        }));
    }

    // ── 2.5) Si turno_actual es NULL o 0, asignarlo usando turno_inicio = 1 ─
    if partida.turno_actual.unwrap_or(0) == 0 {
        if let Some(jugador_inicia) = formaciones.iter().find(|f| f.turno_inicio == 1) {
            println!("🎯 Asignando turno inicial al jugador: {}", jugador_inicia.id_usuario);
            sqlx::query!(
                "UPDATE Partida SET turno_actual = ? WHERE id_partida = ?",
                jugador_inicia.id_usuario,
                id_partida
            )
                .execute(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            partida.turno_actual = Some(jugador_inicia.id_usuario);
        }
    }

    // ── 3-a) marcador ───────────────────────────────────────────────────
    let marcador = sqlx::query!(
        "SELECT gol_j1, gol_j2 FROM Partida WHERE id_partida = ?",
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ── 3-b) turnos ─────────────────────────────────────────────────────
    let turnos: Vec<TurnoData> = sqlx::query_as!(
        TurnoData,
        r#"
        SELECT
            numero_turno,
            id_usuario,
            jugada,
            fecha_turno AS "fecha_turno: chrono::NaiveDateTime"
        FROM Turno
        WHERE id_partida = ?
        ORDER BY numero_turno
        "#,
        id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let turno_final = partida.turno_actual.unwrap_or(0);
    println!("📦 Snapshot final → turno_actual = {turno_final}");

    // ── 4) devolver snapshot completo ───────────────────────────────────
    Ok(Json(Snapshot {
        marcador: (
            marcador.gol_j1.unwrap_or(0),
            marcador.gol_j2.unwrap_or(0),
        ),
        formaciones,
        turnos,
        proximo_turno: turno_final,
    }))
}


#[axum::debug_handler]
pub async fn get_partidas_pendientes(
    Path(id_usuario): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Vec<Partida>>, (StatusCode, String)> {
    let partidas = sqlx::query_as!(
        Partida,
        r#"
        SELECT
            id_partida,
            id_jugador1 AS id_usuario_1,
            id_jugador2 AS id_usuario_2,
            fecha_inicio AS "fecha_creacion: chrono::NaiveDateTime",
            estado
        FROM Partida
        WHERE estado = 'waiting'
          AND (id_jugador1 = ? OR id_jugador2 = ?)
          AND id_partida NOT IN (
              SELECT id_partida FROM FormacionElegida WHERE id_usuario = ?
          )
        "#,
        id_usuario,
        id_usuario,
        id_usuario
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(partidas))
}

#[axum::debug_handler]
pub async fn get_partida_detalle(
    Path(id): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Partida>, (StatusCode, String)> {
    let row = sqlx::query_as!(
        Partida,
        r#"
        SELECT id_partida,
               id_jugador1 AS id_usuario_1,
               id_jugador2 AS id_usuario_2,
               fecha_inicio AS "fecha_creacion: chrono::NaiveDateTime",
               estado
        FROM Partida
        WHERE id_partida = ?
        "#,
        id
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(row))
}



