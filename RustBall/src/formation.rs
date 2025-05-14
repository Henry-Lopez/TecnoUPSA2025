use bevy::prelude::*;
use crate::resources::Formation;

/// Devuelve las posiciones para una formación, reflejadas según el lado del jugador
pub fn get_formation_positions(formation: Formation, is_left: bool) -> Vec<Vec2> {
    let flip = if is_left { -1.0 } else { 1.0 };

    match formation {
        Formation::Rombo1211 => vec![
            Vec2::new(400.0, 0.0),
            Vec2::new(300.0, 100.0),
            Vec2::new(300.0, -100.0),
            Vec2::new(200.0, 0.0),
            Vec2::new(100.0, 0.0),
        ],
        Formation::Muro221 => vec![
            Vec2::new(400.0, 100.0),
            Vec2::new(400.0, -100.0),
            Vec2::new(250.0, 100.0),
            Vec2::new(250.0, -100.0),
            Vec2::new(100.0, 0.0),
        ],
        Formation::Ofensiva113 => vec![
            Vec2::new(300.0, 150.0),
            Vec2::new(300.0, 0.0),
            Vec2::new(300.0, -150.0),
            Vec2::new(200.0, 0.0),
            Vec2::new(400.0, 0.0),
        ],
        Formation::Diamante2111 => vec![
            Vec2::new(400.0, 100.0),
            Vec2::new(400.0, -100.0),
            Vec2::new(300.0, 0.0),
            Vec2::new(200.0, 0.0),
            Vec2::new(100.0, 0.0),
        ],
    }
        .into_iter()
        .map(|v| Vec2::new(v.x * flip, v.y))
        .collect()
}
