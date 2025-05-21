use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{
    components::{PlayerDisk, OwnedBy},
    snapshot::BoardSnapshot,
};

pub fn apply_board_snapshot(board: BoardSnapshot, commands: &mut Commands) {
    let damping = Damping {
        linear_damping: 2.0,
        angular_damping: 2.0,
    };

    for pieza in board.piezas {
        let player_id = pieza.id as i32;
        let user_id = pieza.id_usuario_real;

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
                    ..default()
                },
                ..default()
            },
            RigidBody::Dynamic,
            Collider::ball(35.0),
            Restitution::coefficient(0.5),
            ActiveEvents::COLLISION_EVENTS,
            ExternalImpulse::default(),
            ExternalForce::default(),
            AdditionalMassProperties::Mass(1.0),
            Velocity::zero(),
            damping.clone(),
            LockedAxes::ROTATION_LOCKED,
            Sleeping::disabled(),
            PlayerDisk {
                player_id,
                id_usuario_real: user_id,
            },
            OwnedBy(user_id),
            Name::new(format!("disk_user_{}", user_id)),
        ));
    }
}
