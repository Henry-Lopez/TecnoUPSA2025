use bevy::prelude::*;

/// Disco de jugador
// components.rs
#[derive(Component)]
pub struct PlayerDisk {
 pub player_id: i32,
 pub id_usuario_real: i32,
}


/// Pelota
#[derive(Component)]
pub struct Ball;

/// Componente marcador de turno
#[derive(Component)]
pub struct TurnControlled;

/// Zona de gol
#[derive(Component)]
pub struct GoalZone {
 pub is_left: bool,
}

/// Texto animado del power-up (solo si lo dejás con parpadeo)

/// Componente que identifica al texto de turno
#[derive(Component)]
pub struct TurnText;

/// Componente que identifica al texto de score
#[derive(Component)]
pub struct ScoreText;

/// Barra de poder visual
#[derive(Component)]
pub struct PowerBar;

/// Menú de formaciones
#[derive(Component)]
pub struct FormationMenu;

/// Texto del power-up encima de cada jugador
#[derive(Component)]
pub struct PowerUpLabel;

/// Componente opcional si usás visibilidad en vez de despawn
#[derive(Component)]
pub struct PowerUpLabelVisibility;

#[derive(Component)]
pub struct GameplayEntity;   // marcador vacío

#[derive(Component)]
pub struct OwnedBy(pub i32);  // 👤 ID real del dueño de la ficha
