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
    Extension(tx):   Extension<broadcast::Sender<String>>,
    Json(payload):   Json<JugadaPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    tracing::info!("▶️  POST /jugada  —  {:?}", payload);

    /* 1. Validar turno_actual ----------------------------------------- */
    tracing::debug!("🔍 Leyendo turno_actual de BD…");
    let turno_actual: Option<i32> = sqlx::query_scalar!(
        "SELECT turno_actual FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error turno_actual: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    tracing::debug!(
        turno_actual = ?turno_actual,
        id_usuario   = payload.id_usuario,
        "validando turno"
    );

    if turno_actual != Some(payload.id_usuario) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("No es el turno del usuario {}.", payload.id_usuario),
        ));
    }

    /* ⚠  Desde aquí todo en el mismo orden (opcionalmente TRANSACTION) */
    /* 2. Calcular numero_turno --------------------------------------- */
    let max_turno_i64: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(numero_turno),0) FROM Turno WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error MAX(numero_turno): {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let nuevo_turno = (max_turno_i64 as i32) + 1;
    tracing::debug!(max_turno = max_turno_i64, nuevo_turno, "calculado");

    /* 3. Enriquecer jugada con id_usuario_real ----------------------- */
    let piezas_json = payload
        .jugada
        .get("piezas")
        .and_then(|v| v.as_array())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Formato de jugada inválido: falta 'piezas'".to_string(),
        ))?;

    let jugada_json = json!({
        "piezas": piezas_json.iter().map(|p| {
            json!({
                "id_usuario_real": payload.id_usuario,
                "x": p["x"],
                "y": p["y"]
            })
        }).collect::<Vec<_>>()
    });

    /* 4. Insertar turno ---------------------------------------------- */
    sqlx::query!(
        r#"
        INSERT INTO Turno (id_partida, numero_turno, id_usuario, jugada)
        VALUES (?, ?, ?, ?)
        "#,
        payload.id_partida,
        nuevo_turno,
        payload.id_usuario,
        jugada_json
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            let dup = e
                .as_database_error()
                .and_then(|d| d.code())
                .map(|c| c == "1062")
                .unwrap_or(false);

            if dup {
                (StatusCode::CONFLICT, "Número de turno duplicado".into())
            } else {
                tracing::error!("❌ SQL error INSERT Turno: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;
    tracing::debug!("✅ Turno #{nuevo_turno} insertado");

    /* 5. Calcular siguiente jugador ---------------------------------- */
    let (j1, j2) = sqlx::query!(
        "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&pool)
        .await
        .map(|r| (r.id_jugador1, r.id_jugador2))
        .map_err(|e| {
            tracing::error!("❌ SQL error jugadores: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let siguiente_turno = if payload.id_usuario == j1 { j2 } else { j1 };

    sqlx::query!(
        "UPDATE Partida SET turno_actual = ? WHERE id_partida = ?",
        siguiente_turno,
        payload.id_partida
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ UPDATE turno_actual: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    tracing::debug!("🔄 turno_actual -> {siguiente_turno}");

    /* 6. Generar snapshot + notificar -------------------------------- */
    let snap = super::get_snapshot(
        Path(payload.id_partida),
        Extension(pool.clone()),
    )
        .await
        .map_err(|e| {
            tracing::error!("❌ Snapshot error: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Error generando snapshot".into())
        })?
        .0;

    let _ = tx.send("turno_finalizado".to_string());
    let _ = tx.send(serde_json::to_string(&snap).unwrap());

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

// 5. POST /formacion ───────────────────────────────────────────────
#[axum::debug_handler]
pub async fn post_formacion(
    Extension(pool): Extension<MySqlPool>,
    Extension(tx):   Extension<broadcast::Sender<String>>,
    Json(p):         Json<FormacionPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    tracing::info!("▶️  POST /formacion — {:?}", p);

    /* 1. INSERT / UPDATE ------------------------------------------------ */
    tracing::info!("1️⃣  Guardando formación…");
    sqlx::query!(
        r#"
        INSERT INTO FormacionElegida (id_partida, id_usuario, formacion, turno_inicio)
        VALUES (?, ?, ?, 0)
        ON DUPLICATE KEY UPDATE formacion = VALUES(formacion)
        "#,
        p.id_partida,
        p.id_usuario,
        p.formacion
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error INSERT/UPDATE FormacionElegida: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    /* 2. ¿Ya hay 2 formaciones? ---------------------------------------- */
    tracing::info!("2️⃣  Comprobando si ya hay 2 formaciones…");
    let formaciones = sqlx::query!(
        "SELECT id_usuario, turno_inicio FROM FormacionElegida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error SELECT FormacionElegida: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if formaciones.len() < 2 {
        tracing::info!("ℹ️  Falta la otra formación (len={})", formaciones.len());
        return Ok(Json("Formación registrada"));
    }

    /* 3. Calcular turno_inicio=1  (el que arranca) --------------------- */
    let primero = formaciones
        .iter()
        .find(|f| f.turno_inicio == 1)
        .map(|f| f.id_usuario);

    let primero = match primero {
        Some(uid) => uid,
        None => {
            // Si aún no se inicializó turno_inicio, sorteamos y actualizamos.
            let [a, b] = [formaciones[0].id_usuario, formaciones[1].id_usuario];
            let (primero, segundo) = if rand::random() { (a, b) } else { (b, a) };

            for (uid, idx) in [(primero, 1), (segundo, 2)] {
                sqlx::query!(
                    "UPDATE FormacionElegida SET turno_inicio = ? WHERE id_partida = ? AND id_usuario = ?",
                    idx,
                    p.id_partida,
                    uid
                )
                    .execute(&pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("❌ UPDATE turno_inicio (uid={uid}): {e:?}");
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    })?;
            }
            primero
        }
    };
    tracing::info!("3️⃣  turno_actual inicial será uid={primero}");

    /* 4. UPDATE Partida → estado='playing', turno_actual -------------- */
    sqlx::query!(
        "UPDATE Partida SET estado = 'playing', turno_actual = ? WHERE id_partida = ?",
        primero,
        p.id_partida
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ UPDATE Partida: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    /* 5. Generar snapshot inicial y avisar ---------------------------- */
    tracing::info!("5️⃣  Generando snapshot inicial…");
    let snap = super::get_snapshot(
        Path(p.id_partida),
        Extension(pool.clone()),
    )
        .await
        .map_err(|e| {
            tracing::error!("❌ Error generando snapshot: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Error generando snapshot".into())
        })?
        .0;

    // Mensaje 'start' + snapshot completo
    let _ = tx.send("start".to_string());
    let _ = tx.send(
        serde_json::to_string(&snap).expect("snapshot serializable"),
    );
    tracing::info!("📡 Snapshot inicial + 'start' enviados");

    Ok(Json("Formación registrada y partida arrancada"))
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
// 🔧 Handler limpio y sin #[debug_handler]
#[axum::debug_handler]
pub async fn get_snapshot(
    Path(id_partida): Path<i32>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<Json<Snapshot>, (StatusCode, String)> {
    // ───── 0) Estado de la partida ─────────────────────────────────────
    let mut _partida = sqlx::query!(
        r#"
        SELECT estado AS "estado!: String", turno_actual
        FROM Partida
        WHERE id_partida = ?
        "#,
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ───── 1) Nombres de jugadores ────────────────────────────────────
    let nombres = sqlx::query!(
        r#"
        SELECT u1.nombre_usuario AS nombre_jugador_1,
               u2.nombre_usuario AS nombre_jugador_2
        FROM   Partida
        JOIN   Usuario u1 ON u1.id_usuario = Partida.id_jugador1
        JOIN   Usuario u2 ON u2.id_usuario = Partida.id_jugador2
        WHERE  Partida.id_partida = ?
        "#,
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ───── 2) Formaciones elegidas ────────────────────────────────────
    let formaciones = sqlx::query_as!(
        FormacionData,
        r#"
        SELECT id_usuario, formacion, turno_inicio
        FROM   FormacionElegida
        WHERE  id_partida = ?
        "#,
        id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Mientras no haya 2 formaciones devolvemos un snapshot “vacío”
    if formaciones.len() < 2 {
        return Ok(Json(Snapshot {
            marcador: (0, 0),
            formaciones,
            turnos: vec![],
            proximo_turno: 0,
            nombre_jugador_1: nombres.nombre_jugador_1,
            nombre_jugador_2: nombres.nombre_jugador_2,
        }));
    }

    // ───── 3) Asegurar que turno_actual esté definido ─────────────────
    if _partida.turno_actual.is_none() {
        if let Some(j1) = formaciones.iter().find(|f| f.turno_inicio == 1) {
            sqlx::query!(
                "UPDATE Partida SET turno_actual = ? WHERE id_partida = ? AND turno_actual IS NULL",
                j1.id_usuario,
                id_partida
            )
                .execute(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            _partida.turno_actual = Some(j1.id_usuario);
        }
    }

    // ───── 4) Marcador actual ─────────────────────────────────────────
    let marcador = sqlx::query!(
        "SELECT gol_j1, gol_j2 FROM Partida WHERE id_partida = ?",
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ───── 5) Turnos (¡los enriquecemos aquí!) ───────────────────────
    let mut turnos: Vec<TurnoData> = sqlx::query_as!(
        TurnoData,
        r#"
        SELECT numero_turno,
               id_usuario,
               jugada,
               fecha_turno AS "fecha_turno: chrono::NaiveDateTime"
        FROM   Turno
        WHERE  id_partida = ?
        ORDER  BY numero_turno
        "#,
        id_partida
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ► Añadimos id_usuario_real a cada pieza
    for t in &mut turnos {
        if let Some(arr) = t.jugada.get("piezas").and_then(|v| v.as_array()) {
            let enriched: Vec<_> = arr
                .iter()
                .map(|p| {
                    json!({
                        "id_usuario_real": t.id_usuario,   // ← dueño real
                        "x": p["x"],
                        "y": p["y"]
                    })
                })
                .collect();

            t.jugada = json!({ "piezas": enriched });
        }
    }

    // ───── 6) Construir y devolver el snapshot completo ───────────────
    Ok(Json(Snapshot {
        marcador: (
            marcador.gol_j1.unwrap_or(0),
            marcador.gol_j2.unwrap_or(0),
        ),
        formaciones,
        turnos,
        proximo_turno: _partida.turno_actual.unwrap_or(0),
        nombre_jugador_1: nombres.nombre_jugador_1,
        nombre_jugador_2: nombres.nombre_jugador_2,
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



