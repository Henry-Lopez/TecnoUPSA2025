use bevy::prelude::*;
use crate::events::RandomEvent;
use crate::snapshot::SnapshotFromServer;

/* ─────────────── Turno actual del juego ─────────────── */
// resources.rs
#[derive(Resource)] // ⬅️ esto es lo que falta
pub struct TurnState {
    pub current_turn_id: i32,  // 👈 nuevo nombre correcto
    pub in_motion: bool,
    pub selected_entity: Option<Entity>,
    pub aim_direction: Vec2,
    pub power: f32,
    pub skip_turn_switch: bool,
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_turn_id: 0,
            in_motion: false,
            selected_entity: None,
            aim_direction: Vec2::ZERO,
            power: 0.0,
            skip_turn_switch: false,
        }
    }
}


/* ─────────────── Puntos de cada lado ─────────────── */
#[derive(Resource, Default)]
pub struct Scores {
    pub left: u32,
    pub right: u32,
}

/* ─────────────── Formaciones disponibles ─────────────── */
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Formation {
    Rombo1211,
    Muro221,
    Ofensiva113,
    Diamante2111,
}

impl Formation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Formation::Rombo1211    => "1-2-1-1",
            Formation::Muro221      => "2-2-1",
            Formation::Ofensiva113  => "1-1-3",
            Formation::Diamante2111 => "2-1-1-1",
        }
    }
}

/* ─────────────── Formaciones seleccionadas por cada jugador ─────────────── */
#[derive(Resource, Debug)]
pub struct PlayerFormations {
    pub player1: Option<Formation>,
    pub player2: Option<Formation>,
}

/* ─────────────── Estados del juego ─────────────── */
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    FormationSelection,
    InGame,
    GoalScored,
    FormationChange,
    GameOver,
}

/* ─────────────── Imagen del fondo final ─────────────── */
#[derive(Resource)]
pub struct GameOverBackground(pub Handle<Image>);

/* ─────────────── Eventos aleatorios ─────────────── */
#[derive(Resource, Default)]
pub struct EventControl {
    pub turns_since_last: usize,
    pub current_event: Option<RandomEvent>,
    pub event_active: bool,
}

/* ─────────────── Info del backend para identificar jugadores ─────────────── */
#[derive(Resource, Debug, Clone)]
pub struct BackendInfo {
    pub partida_id: i32,
    pub my_uid:     i32,
    pub id_left:    i32,
    pub id_right:   i32,
}

impl BackendInfo {
    pub fn new(partida_id: i32, my_uid: i32, id_left: i32, id_right: i32) -> Self {
        Self { partida_id, my_uid, id_left, id_right }
    }

    pub fn i_am_left(&self) -> bool {
        self.my_uid == self.id_left
    }

    pub fn i_am_right(&self) -> bool {
        self.my_uid == self.id_right
    }
}

/* ─────────────── Snapshot más reciente recibido ─────────────── */
#[derive(Resource, Default)]
pub struct LatestSnapshot(pub Option<SnapshotFromServer>);

#[derive(Component)]
pub struct PowerBarBackground;

#[derive(Resource, Default)]
pub struct WsInbox(pub Vec<String>);

#[derive(Resource, Default)]
pub struct UltimoTurnoAplicado(pub i32);

#[derive(Resource)]
pub struct CurrentPlayerId(pub i32);
