use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JugadaPayload {
    pub id_partida: i32,
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TurnoData {
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: Value,
    pub fecha_turno: Option<NaiveDateTime>,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Usuario {
    pub id_usuario: i32,
    pub nombre_usuario: String,
    pub correo: String,
    pub contrasena: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Estadistica {
    pub id_usuario: i32,
    pub partidas_jugadas: Option<i32>,
    pub partidas_ganadas: Option<i32>,
    pub goles_a_favor: Option<i32>,
    pub goles_en_contra: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormacionPayload {
    pub id_partida: i32,
    pub id_usuario: i32,
    pub formacion: String,
    pub turno_inicio: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistroPayload {
    pub nombre_usuario: String,
    pub correo: String,
    pub contrasena: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartidaPayload {
    pub id_usuario_1: i32,
    pub id_usuario_2: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Partida {
    pub id_partida:  i32,
    pub id_usuario_1: i32,
    pub id_usuario_2: i32,
    pub fecha_creacion: Option<NaiveDateTime>,   // ← otra vez NaiveDateTime
    pub estado: String,
}

#[derive(serde::Deserialize)]
pub struct GolPayload {
    pub id_partida:  i32,
    pub id_goleador: i32,   // jugador que anotó
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormacionData {
    pub id_usuario: i32,
    pub formacion : String,
    pub turno_inicio: i32,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub marcador: (i32, i32),
    pub formaciones: Vec<FormacionData>,
    pub turnos: Vec<TurnoData>,
    pub proximo_turno: Option<i32>, // ← CAMBIO AQUÍ
    pub nombre_jugador_1: String,
    pub nombre_jugador_2: String,
}

