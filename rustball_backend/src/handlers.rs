    // ─────────────────────────────────────────────────────────────
    // handlers.rs  (al principio del fichero, antes de cualquier use o fn)
    // ─────────────────────────────────────────────────────────────
    // handlers.rs  ── ANTES de cualquier `use` o `fn`
    macro_rules! internal {
        ($ctx:literal) => {
            |e| {
                // e implementa Debug siempre, Display no siempre
                tracing::error!("❌ Error al {}: {:?}", $ctx, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error al {}: {:?}", $ctx, e),
                )
            }
        };
    }

    // Si lo pones aquí, todos los handlers ya lo tienen disponible.

        use axum::{
            extract::{Extension, Path},
            http::StatusCode,
            Json,
        };
        use serde::Deserialize;
        use serde_json::json; // Asegúrate de que esto está importado
        use sqlx::{MySqlPool}; // Mantendremos las importaciones que tenías si las quieres
        use tokio::sync::broadcast;
        use crate::models::*; // Asegúrate de que tus modelos están en scope
        use tracing; // Asegúrate de que tracing está en scope

    #[axum::debug_handler]
    pub async fn post_jugada(
        Extension(pool): Extension<MySqlPool>,
        Extension(tx):   Extension<broadcast::Sender<String>>,
        Json(payload):   Json<JugadaPayload>,
    ) -> Result<Json<&'static str>, (StatusCode, String)> {
        tracing::info!("▶️  POST /jugada — Recibido payload: {:?}", payload);

        /* ───────────────────────── 1. VALIDAR + INSERTAR ───────────────────────── */
        let mut tx_db = pool
            .begin()
            .await
            .map_err(internal!("iniciar transacción"))?;

        // 1-A) comprobar que es su turno
        let turno_actual: Option<i32> = sqlx::query_scalar!(
                "SELECT turno_actual FROM Partida WHERE id_partida = ?",
                payload.id_partida
            )
            .fetch_one(&mut *tx_db)
            .await
            .map_err(internal!("leer turno_actual"))?;

        if turno_actual != Some(payload.id_usuario) {
            tracing::warn!("⛔ usuario fuera de turno ({:?})", turno_actual);
            return Err((StatusCode::BAD_REQUEST, "No es tu turno".into()));
        }

        // 1-B) nuevo número de turno (devuelve i64 → cast a i32)
        let nuevo_turno_i64: i64 = sqlx::query_scalar!(
                "SELECT COALESCE(MAX(numero_turno),0)+1 FROM Turno WHERE id_partida = ?",
                payload.id_partida
            )
            .fetch_one(&mut *tx_db)
            .await
            .map_err(internal!("calcular numero_turno"))?;

        let nuevo_turno: i32 = nuevo_turno_i64 as i32;

        // 1-C) INSERT idempotente
        sqlx::query!(
            r#"
            INSERT INTO Turno (id_partida, numero_turno, id_usuario, jugada)
            VALUES (?,?,?,?)
            ON DUPLICATE KEY UPDATE jugada = VALUES(jugada)
            "#,
            payload.id_partida,
            nuevo_turno,
            payload.id_usuario,
            payload.jugada
        )
            .execute(&mut *tx_db)
            .await
            .map_err(internal!("insertar turno"))?;

        // 1-D) pasar turno al rival
        let (j1, j2) = sqlx::query!(
                "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
                payload.id_partida
            )
            .fetch_one(&mut *tx_db)
            .await
            .map(|r| (r.id_jugador1, r.id_jugador2))
            .map_err(internal!("leer jugadores"))?;

        let siguiente_turno = if payload.id_usuario == j1 { j2 } else { j1 };

        sqlx::query!(
            "UPDATE Partida SET turno_actual = ? WHERE id_partida = ?",
            siguiente_turno,
            payload.id_partida
        )
            .execute(&mut *tx_db)
            .await
            .map_err(internal!("actualizar turno_actual"))?;

        tx_db
            .commit()
            .await
            .map_err(internal!("confirmar transacción"))?;

        /* ───────────────────────── 2. ENVIAR SNAPSHOT ───────────────────────── */
        let snap = super::get_snapshot(payload.id_partida, pool.clone())
            .await
            .map_err(internal!("generar snapshot"))?;

        let ws_msg = serde_json::json!({
            "uid_origen": payload.id_usuario,
            "tipo"      : "snapshot",
            "contenido" : snap
        });

        if let Err(e) = tx.send(ws_msg.to_string()) {
            tracing::warn!("📢 No hay oyentes para snapshot: {e}");
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

            let primero = formaciones_para_tx
                .iter()
                .find(|f| f.turno_inicio == 1)
                .map(|f| f.id_usuario);

            let primero = match primero {
                Some(uid) => uid,
                None => {
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
                .execute(&mut *transaction)
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
            let snap = super::get_snapshot(p.id_partida, pool.clone())
                .await
                .map_err(|e| {
                    tracing::error!("❌ Error generando snapshot: {e:?}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Error generando snapshot".into())
                })?;

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

    // -----------------------------------------------------------------------------
    //  POST /gol
    // -----------------------------------------------------------------------------
    #[axum::debug_handler]
    pub async fn post_gol(
        Extension(pool): Extension<MySqlPool>,
        Extension(tx):   Extension<broadcast::Sender<String>>,   // ⬅️  añade el sender
        Json(p):         Json<GolPayload>,
    ) -> Result<Json<(i32, i32)>, (StatusCode, String)> {
        tracing::info!("⚽  POST /gol — partida {}, goleador {}", p.id_partida, p.id_goleador);

        /* ───────────────────────── 1. Sumar el gol ───────────────────────── */
        let row = sqlx::query!(
        "SELECT id_jugador1, id_jugador2 FROM Partida WHERE id_partida = ?",
        p.id_partida
    )
            .fetch_one(&pool)
            .await
            .map_err(internal!("leer jugadores de la partida"))?;

        if p.id_goleador == row.id_jugador1 {
            sqlx::query!(
            "UPDATE Partida SET gol_j1 = gol_j1 + 1 WHERE id_partida = ?",
            p.id_partida
        )
                .execute(&pool)
                .await
                .map_err(internal!("incrementar gol_j1"))?;
        } else {
            sqlx::query!(
            "UPDATE Partida SET gol_j2 = gol_j2 + 1 WHERE id_partida = ?",
            p.id_partida
        )
                .execute(&pool)
                .await
                .map_err(internal!("incrementar gol_j2"))?;
        }

        /* ───────────────────────── 2. Resetear la partida ──────────────────
           - estado  -> 'waiting'
           - turno_actual -> 0
           - borrar formaciones elegidas     */
        sqlx::query!(
        "UPDATE Partida
            SET estado = 'waiting',
                turno_actual = 0
          WHERE id_partida = ?",
        p.id_partida
    )
            .execute(&pool)
            .await
            .map_err(internal!("poner estado=waiting"))?;

        sqlx::query!(
        "DELETE FROM FormacionElegida WHERE id_partida = ?",
        p.id_partida
    )
            .execute(&pool)
            .await
            .map_err(internal!("borrar formaciones"))?;

        /* ───────────────────────── 3. Consultar marcador ─────────────────── */
        let marcador = sqlx::query!(
        "SELECT gol_j1, gol_j2 FROM Partida WHERE id_partida = ?",
        p.id_partida
    )
            .fetch_one(&pool)
            .await
            .map_err(internal!("leer marcador"))?;

        /* ───────────────────────── 4. Enviar snapshot 'waiting' ───────────── */
        let snap = super::get_snapshot(p.id_partida, pool.clone())
            .await
            .map_err(internal!("generar snapshot"))?;

        if let Err(e) = tx.send(serde_json::to_string(&snap).unwrap()) {
            tracing::warn!("📢 No hay oyentes para snapshot post-gol: {e}");
        }

        /* ───────────────────────── 5. Respuesta HTTP ─────────────────────── */
        Ok(Json((
            marcador.gol_j1.unwrap_or(0),
            marcador.gol_j2.unwrap_or(0),
        )))
    }

    use crate::routes::websocket::save_last_snapshot; // 🆕 Agrega este import

    // -----------------------------------------------------------------------------
    //  GET /snapshot/{id_partida}
    // -----------------------------------------------------------------------------
    pub async fn get_snapshot(
        id_partida: i32,
        pool: MySqlPool,
    ) -> Result<Snapshot, (StatusCode, String)> {
        tracing::info!("▶️ Generando snapshot de partida {id_partida}");

        /* ───── 1. Cabecera de la partida ───── */
        let partida_data = sqlx::query!(
        r#"
        SELECT estado  AS "estado!: String",
               turno_actual,
               gol_j1,
               gol_j2
        FROM   Partida
        WHERE  id_partida = ?
        "#,
        id_partida
    )
            .fetch_one(&pool)
            .await
            .map_err(|e| internal!("estado de la partida")(e))?;

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
            .map_err(|e| internal!("nombres de jugadores")(e))?;

        /* ───── 2. Formaciones ───── */
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
            .map_err(|e| internal!("formaciones")(e))?;

        /* ── 2-bis. Último nº de turno real ── */
        let ultimo_turno_i64: i64 = sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(numero_turno), 0)
           FROM   Turno
           WHERE  id_partida = ?"#,
        id_partida
    )
            .fetch_one(&pool)
            .await
            .map_err(internal!("último nº de turno"))?;   // 👈 usa tu macro

        let ultimo_turno: i32 = ultimo_turno_i64 as i32;   // 👈 conversión explícita


        // Si no hay las 2 formaciones devolvemos snapshot mínimo
        if formaciones.len() < 2 {
            let snapshot = Snapshot {
                estado: "waiting".into(),
                marcador: (0, 0),
                formaciones,
                turnos: vec![],
                proximo_turno: 0,
                ultimo_turno,                       // ← ya incluido
                nombre_jugador_1: nombres.nombre_jugador_1,
                nombre_jugador_2: nombres.nombre_jugador_2,
            };
            if let Ok(s) = serde_json::to_string(&snapshot) {
                save_last_snapshot(id_partida, s);
            }
            return Ok(snapshot);
        }

        /* ───── 3. Turnos y jugadas ───── */
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
            .map_err(|e| internal!("turnos")(e))?;

        // ➜ Enriquecer cada jugada
        for t in &mut turnos {
            if let Some(arr) = t.jugada.get("piezas").and_then(|v| v.as_array()) {
                let enriched: Vec<_> = arr
                    .iter()
                    .map(|p| {
                        let id_val = p.get("id").cloned().unwrap_or(json!(null));
                        let owner = p
                            .get("id_usuario_real")
                            .cloned()
                            .unwrap_or_else(|| {
                                if id_val == json!(-1) { json!(0) } else { json!(t.id_usuario) }
                            });

                        json!({
                        "id"             : id_val,
                        "id_usuario_real": owner,
                        "x"              : p.get("x").cloned().unwrap_or(json!(null)),
                        "y"              : p.get("y").cloned().unwrap_or(json!(null)),
                    })
                    })
                    .collect();

                t.jugada = json!({ "piezas": enriched });
            } else {
                tracing::warn!(
                "⚠️ Turno #{} sin piezas válidas (jugada original = {:?})",
                t.numero_turno,
                t.jugada
            );
            }
        }

        /* ───── 4. Empaquetar snapshot completo ───── */
        let snapshot = Snapshot {
            estado: "playing".into(),
            marcador: (
                partida_data.gol_j1.unwrap_or(0),
                partida_data.gol_j2.unwrap_or(0),
            ),
            formaciones,
            turnos,
            proximo_turno: partida_data.turno_actual.unwrap_or(0),
            ultimo_turno,                           // ← NUEVO campo
            nombre_jugador_1: nombres.nombre_jugador_1,
            nombre_jugador_2: nombres.nombre_jugador_2,
        };

        if let Ok(s) = serde_json::to_string(&snapshot) {
            save_last_snapshot(id_partida, s);
        }

        tracing::info!("✅ Snapshot de partida {id_partida} generado");
        Ok(snapshot)
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



