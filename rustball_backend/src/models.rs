use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JugadaPayload {
    pub id_partida: i32,
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: Value,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
    pub id_partida: i32,
    pub id_usuario_1: i32,
    pub id_usuario_2: i32,
    pub fecha_creacion: Option<NaiveDateTime>,
}