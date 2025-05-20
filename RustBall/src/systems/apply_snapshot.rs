use bevy::prelude::*;
use crate::{
    components::PlayerDisk,
    snapshot::BoardSnapshot,
};

/// Recoloca cada disco según la foto enviada desde el backend.
/// Usa `player_id` para lado (1 = izquierda, 2 = derecha) y `id_usuario_real` para personalización.
pub fn apply_board_snapshot(board: BoardSnapshot, commands: &mut Commands) {
    for pieza in board.piezas {
        let player_id = pieza.id as usize;
        let user_id = pieza.id_usuario_real;

        // 🎨 Puedes personalizar colores según el usuario
        let color = match user_id {
            3 => Color::BLUE,
            4 => Color::ORANGE,
            5 => Color::GREEN,
            _ => Color::WHITE,
        };

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(pieza.x, pieza.y, 10.0),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(70.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            PlayerDisk {
                player_id,
                id_usuario_real: user_id,
            },
            Name::new(format!("disk_user_{}", user_id)),
        ));
    }
}
