use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use futures_util::TryFutureExt;
use serde::Deserialize;
use serde_json::json; // Asegúrate de que esto está importado
use sqlx::{MySqlPool}; // Mantendremos las importaciones que tenías si las quieres
use tokio::sync::broadcast;
use crate::models::*; // Asegúrate de que tus modelos están en scope
use tracing; // Asegúrate de que tracing está en scope

#[axum::debug_handler]
pub async fn post_jugada(
    Extension(pool): Extension<MySqlPool>,
    Extension(tx): Extension<broadcast::Sender<String>>,
    Json(payload): Json<JugadaPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    tracing::info!("▶️  POST /jugada — Recibido payload: {:?}", payload);

    // Iniciar una transacción
    let mut transaction = pool.begin().await.map_err(|e| {
        tracing::error!("❌ Error al iniciar transacción: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al iniciar transacción: {}", e))
    })?;

    /* ─── 1. Validar turno_actual ───────────────────────────── */
    tracing::debug!("🔍 Leyendo turno_actual desde la base de datos...");
    let turno_actual: Option<i32> = sqlx::query_scalar!(
        "SELECT turno_actual FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al consultar turno_actual: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al leer turno_actual: {}", e))
        })?;

    tracing::debug!(?turno_actual, "✅ turno_actual leído correctamente");

    if turno_actual != Some(payload.id_usuario) {
        tracing::warn!(
            "⛔ Jugador {} intentó jugar fuera de turno. Turno actual: {:?}",
            payload.id_usuario,
            turno_actual
        );
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "No es el turno del usuario {}. Turno actual: {:?}",
                payload.id_usuario, turno_actual
            ),
        ));
    }

    /* ─── 2. Calcular nuevo número de turno ─────────────────── */
    let max_turno_i64: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(numero_turno), 0) FROM Turno WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al calcular MAX(numero_turno): {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error interno al calcular número de turno".into())
        })?;

    let nuevo_turno = (max_turno_i64 as i32) + 1;
    tracing::debug!(nuevo_turno, "✅ Nuevo número de turno calculado");

    /* ─── 3. Enriquecer jugada con id_usuario_real ───────────── */
    let piezas_json = payload
        .jugada
        .get("piezas")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            tracing::warn!("⚠️ Formato inválido: falta 'piezas' en jugada");
            (
                StatusCode::BAD_REQUEST,
                "Formato de jugada inválido: falta 'piezas'".to_string(),
            )
        })?;

    let jugada_json = json!({
        "piezas": piezas_json.iter().map(|p| {
            json!({
                "id_usuario_real": payload.id_usuario,
                "x": p.get("x").unwrap_or(&json!(null)),
                "y": p.get("y").unwrap_or(&json!(null))
            })
        }).collect::<Vec<_>>()
    });

    tracing::debug!("📦 Jugada enriquecida lista para insertar: {:?}", jugada_json);

    /* ─── 4. Insertar turno ──────────────────────────────────── */
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
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            let is_duplicate = e
                .as_database_error()
                .and_then(|d| d.code())
                .map(|c| c == "1062")
                .unwrap_or(false);

            if is_duplicate {
                tracing::warn!("⚠️ Turno duplicado detectado");
                (StatusCode::CONFLICT, "Número de turno duplicado".into())
            } else {
                tracing::error!("❌ Error al insertar turno: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error insertando turno: {}", e))
            }
        })?;

    tracing::info!("✅ Turno #{nuevo_turno} insertado correctamente");

    /* ─── 5. Calcular siguiente jugador ─────────────────────── */
    let (j1, j2) = sqlx::query!(
        "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
        payload.id_partida
    )
        .fetch_one(&mut *transaction)
        .await
        .map(|r| (r.id_jugador1, r.id_jugador2))
        .map_err(|e| {
            tracing::error!("❌ Error al obtener jugadores: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al obtener jugadores: {}", e))
        })?;

    let siguiente_turno = if payload.id_usuario == j1 { j2 } else { j1 };

    sqlx::query!(
        "UPDATE Partida SET turno_actual = ? WHERE id_partida = ?",
        siguiente_turno,
        payload.id_partida
    )
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al actualizar turno_actual: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al actualizar turno_actual: {}", e))
        })?;

    tracing::info!("🔄 turno_actual actualizado a {}", siguiente_turno);

    /* ─── 6. Confirmar transacción ───────────────────────────── */
    transaction.commit().await.map_err(|e| {
        tracing::error!("❌ Error al confirmar transacción: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al confirmar: {}", e))
    })?;

    tracing::info!("✅ Transacción confirmada correctamente");

    /* ─── 7. Generar snapshot y notificar ───────────────────── */
    let snap = super::get_snapshot(
        Path(payload.id_partida),
        Extension(pool.clone()),
    )
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al generar snapshot: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error generando snapshot".into())
        })?
        .0;

    if tx.send("turno_finalizado".to_string()).is_err() {
        tracing::warn!("📢 No hay oyentes para 'turno_finalizado'");
    }

    if let Err(e) = tx.send(serde_json::to_string(&snap).unwrap_or_default()) {
        tracing::warn!("📢 No hay oyentes para snapshot: {}", e);
    }

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

#[axum::debug_handler]
pub async fn post_formacion(
    Extension(pool): Extension<MySqlPool>,
    Extension(tx):   Extension<broadcast::Sender<String>>,
    Json(p):         Json<FormacionPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    tracing::info!("▶️  POST /formacion — {:?}", p);

    // No iniciamos la transacción de inmediato para permitir que
    // el primer INSERT/UPDATE de `FormacionElegida` se haga fuera si ya existe.
    // Esto es un patrón común: solo empezar la transacción cuando el estado crítico
    // va a ser modificado atómicamente.

    /* 1. INSERT / UPDATE FormacionElegida ------------------------------------------------ */
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
        .execute(&pool) // Usamos el pool directamente aquí
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error INSERT/UPDATE FormacionElegida: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;

    /* 2. ¿Ya hay 2 formaciones? (Lectura fuera de transacción) ---------------------------------------- */
    tracing::info!("2️⃣  Comprobando si ya hay 2 formaciones…");
    let formaciones_existentes = sqlx::query!(
        "SELECT id_usuario, turno_inicio FROM FormacionElegida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_all(&pool) // Usamos el pool directamente para esta lectura
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error SELECT FormacionElegida: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;

    if formaciones_existentes.len() < 2 {
        tracing::info!("ℹ️  Falta la otra formación (len={})", formaciones_existentes.len());
        return Ok(Json("Formación registrada"));
    }

    // ─── A partir de aquí, las operaciones deben ser atómicas. Iniciamos la transacción. ───
    let mut transaction = pool.begin()
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al iniciar transacción: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;

    /* 3. Calcular turno_inicio=1 (el que arranca) --------------------- */
    // Re-leer formaciones dentro de la transacción si la consistencia es ultra-crítica
    // para evitar que otro request las modifique justo antes de la transacción.
    // Para este caso, la lectura fuera de la transacción para el `if formaciones.len() < 2`
    // es probablemente suficiente, ya que si cambia, la transacción lo manejará o fallará.
    // Pero si quieres la máxima seguridad:
    let formaciones_para_tx = sqlx::query!(
        "SELECT id_usuario, turno_inicio FROM FormacionElegida WHERE id_partida = ?",
        p.id_partida
    )
        .fetch_all(&mut *transaction) // Usar &mut *transaction
        .await
        .map_err(|e| {
            tracing::error!("❌ SQL error SELECT FormacionElegida (en TX): {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;


    let primero = formaciones_para_tx // Usamos la re-lectura para la lógica de sorteo
        .iter()
        .find(|f| f.turno_inicio == 1)
        .map(|f| f.id_usuario);

    let primero = match primero {
        Some(uid) => uid,
        None => {
            // Si aún no se inicializó turno_inicio, sorteamos y actualizamos.
            let [a, b] = [formaciones_para_tx[0].id_usuario, formaciones_para_tx[1].id_usuario];
            let (primero, segundo) = if rand::random() { (a, b) } else { (b, a) };

            for (uid, idx) in [(primero, 1), (segundo, 2)] {
                sqlx::query!(
                    "UPDATE FormacionElegida SET turno_inicio = ? WHERE id_partida = ? AND id_usuario = ?",
                    idx,
                    p.id_partida,
                    uid
                )
                    .execute(&mut *transaction) // Usar &mut *transaction
                    .await
                    .map_err(|e| {
                        tracing::error!("❌ UPDATE turno_inicio (uid={uid}) en TX: {e:?}");
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
                    })?;
            }
            primero
        }
    };
    tracing::info!("3️⃣  turno_actual inicial será uid={primero}");

    /* 4. UPDATE Partida → estado='playing', turno_actual (dentro de transacción) -------------- */
    sqlx::query!(
        "UPDATE Partida SET estado = 'playing', turno_actual = ? WHERE id_partida = ?",
        primero,
        p.id_partida
    )
        .execute(&mut *transaction) // Usar &mut *transaction
        .await
        .map_err(|e| {
            tracing::error!("❌ UPDATE Partida en TX: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;
    tracing::debug!("✅ Partida actualizada a 'playing' y turno inicial.");

    // Confirmar la transacción si todo ha ido bien hasta ahora
    transaction.commit()
        .await
        .map_err(|e| {
            tracing::error!("❌ Error al confirmar transacción: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error del servidor: {}", e))
        })?;
    tracing::info!("✅ Transacción de formación confirmada.");

    /* 5. Generar snapshot inicial y avisar (fuera de transacción) ---------------------------- */
    tracing::info!("5️⃣  Generando snapshot inicial…");
    let snap = super::get_snapshot(
        Path(p.id_partida),
        Extension(pool.clone()), // Usar el pool original, no la transacción
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
    tracing::info!("▶️ GET /snapshot/{id_partida} — Solicitando snapshot");

    // ───── 0) Estado de la partida ───────────────────────────────
    let partida_data = sqlx::query!(
        r#"
        SELECT estado AS "estado!: String", turno_actual, gol_j1, gol_j2
        FROM Partida
        WHERE id_partida = ?
        "#,
        id_partida
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Error SQL en estado de partida: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error al obtener estado de la partida".into(),
            )
        })?;

    tracing::debug!(
        estado = %partida_data.estado,
        turno_actual = ?partida_data.turno_actual,
        "ℹ️ Estado inicial de partida obtenido"
    );

    // ───── 1) Nombres de jugadores ────────────────────────────────
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
        .map_err(|e| {
            tracing::error!("❌ Error SQL en nombres de jugadores: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error al obtener nombres de jugadores".into(),
            )
        })?;

    // ───── 2) Formaciones ──────────────────────────────────────────
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
        .map_err(|e| {
            tracing::error!("❌ Error SQL en formaciones: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error al obtener formaciones".into(),
            )
        })?;

    if formaciones.len() < 2 {
        tracing::warn!(
            "⚠️ Solo {} formaciones encontradas. Devolviendo snapshot parcial.",
            formaciones.len()
        );
        return Ok(Json(Snapshot {
            marcador: (0, 0),
            formaciones,
            turnos: vec![],
            proximo_turno: Some(0),
            nombre_jugador_1: nombres.nombre_jugador_1,
            nombre_jugador_2: nombres.nombre_jugador_2,
        }));
    }

    // ───── 3) Marcador ─────────────────────────────────────────────
    let marcador = (
        partida_data.gol_j1.unwrap_or(0),
        partida_data.gol_j2.unwrap_or(0),
    );
    tracing::debug!("📊 Marcador actual: {:?}", marcador);

    // ───── 4) Turnos enriquecidos ─────────────────────────────────
    let mut turnos = sqlx::query_as!(
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
        .map_err(|e| {
            tracing::error!("❌ Error SQL al obtener turnos: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error al obtener turnos".into(),
            )
        })?;

    for t in &mut turnos {
        if let Some(arr) = t.jugada.get("piezas").and_then(|v| v.as_array()) {
            let enriched: Vec<_> = arr
                .iter()
                .map(|p| {
                    json!({
                        "id_usuario_real": t.id_usuario,
                        "x": p.get("x").unwrap_or(&json!(null)),
                        "y": p.get("y").unwrap_or(&json!(null))
                    })
                })
                .collect();
            t.jugada = json!({ "piezas": enriched });
        } else {
            tracing::warn!(
                "⚠️ Turno #{} no tiene piezas válidas. Jugada original: {:?}",
                t.numero_turno,
                t.jugada
            );
        }
    }

    // ───── 5) Retornar Snapshot ─────────────────────────────────────
    tracing::info!("✅ Snapshot de partida {} generado con éxito", id_partida);

    Ok(Json(Snapshot {
        marcador,
        formaciones,
        turnos,
        proximo_turno: partida_data.turno_actual,
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



