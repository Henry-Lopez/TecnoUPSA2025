use bevy::prelude::*;

#[derive(Resource)]          // 👈  quita `Default` aquí
pub struct TurnState {
    pub current_turn: usize,
    pub in_motion: bool,
    pub selected_entity: Option<Entity>,
    pub aim_direction: Vec2,
    pub power: f32,
    pub skip_turn_switch: bool, // 👈 NUEVO
}

// 👇  implementa tu propio Default
impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_turn: 1,          // ⚽ el juego arranca con el jugador 1
            in_motion: false,
            selected_entity: None,
            aim_direction: Vec2::ZERO,
            power: 0.0,
            skip_turn_switch: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct Scores {
    pub left: u32,
    pub right: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Formation {
    Rombo1211,
    Muro221,
    Ofensiva113,
    Diamante2111,
}

#[derive(Resource, Debug)]
pub struct PlayerFormations {
    pub player1: Option<Formation>,
    pub player2: Option<Formation>,
}

/// Estados globales del juego
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    FormationSelection,
    InGame,
    GoalScored,
    FormationChange,
    GameOver, // 👈 nuevo estado
}


#[derive(Component)]
pub struct PowerBarBackground;

#[derive(Resource)]
pub struct GameOverBackground(pub Handle<Image>);

use crate::events::RandomEvent;

#[derive(Resource, Default)]
pub struct EventControl {
    pub turns_since_last: usize,
    pub current_event: Option<RandomEvent>,
    pub event_active: bool, // ✅ Añadí este campo si usás `active`
}


/// Información que el juego necesita para
/// notificar al backend (Axum) eventos como goles.
#[derive(Resource, Debug)]
pub struct BackendInfo {
    pub partida_id: i32,
    pub id_left:    i32,
    pub id_right:   i32,
}
impl BackendInfo {
    pub fn new(partida_id: i32, id_left: i32, id_right: i32) -> Self {
        Self { partida_id, id_left, id_right }
    }
}












