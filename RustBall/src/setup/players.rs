use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::*;
use crate::resources::{PlayerFormations, BackendInfo, TurnState};
use crate::formation::get_formation_positions;

pub fn spawn_players_from_selection(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    formations: Res<PlayerFormations>,
    backend_info: Res<BackendInfo>,
    turn_state: Res<TurnState>,
    existing_players: Query<Entity, With<PlayerDisk>>,
) {
    let damping = Damping {
        linear_damping: 2.0,
        angular_damping: 2.0,
    };

    // 🧹 Elimina jugadores anteriores
    for entity in &existing_players {
        commands.entity(entity).despawn_recursive();
    }

    // 🔵 Jugadores del jugador 1 (izquierda)
    if let Some(f1) = formations.player1 {
        let positions = get_formation_positions(f1, true);
        for (i, pos) in positions.into_iter().enumerate() {
            let mut entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("circulobarca.png"),
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(pos.x, pos.y, 10.0),
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
                    player_id: 1,
                    id_usuario_real: backend_info.id_left,
                },
                                            OwnedBy(backend_info.id_left),
                Name::new(format!("disk_left_{}", i)),
            ));

            // 👉 Si es el jugador activo, da control a su primera ficha
            if backend_info.id_left == backend_info.my_uid
                && backend_info.id_left == turn_state.current_turn_id
                && i == 0
            {
                entity.insert(TurnControlled);
            }
        }
    }

    // 🔴 Jugadores del jugador 2 (derecha)
    if let Some(f2) = formations.player2 {
        let positions = get_formation_positions(f2, false);
        for (i, pos) in positions.into_iter().enumerate() {
            let mut entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("circuloparis.png"),
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(pos.x, pos.y, 10.0),
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
                    player_id: 2,
                    id_usuario_real: backend_info.id_right,
                },
                OwnedBy(backend_info.id_right),
                Name::new(format!("disk_right_{}", i)),
            ));

            if backend_info.id_right == backend_info.my_uid
                && backend_info.id_right == turn_state.current_turn_id
                && i == 0
            {
                entity.insert(TurnControlled);
            }
        }
    }
}
