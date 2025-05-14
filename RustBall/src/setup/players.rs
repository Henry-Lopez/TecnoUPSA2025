use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::*;
use crate::resources::{PlayerFormations};
use crate::formation::get_formation_positions;

pub fn spawn_players_from_selection(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    formations: Res<PlayerFormations>,
    existing_players: Query<Entity, With<PlayerDisk>>, // ✅ añadimos query
) {
    let damping = Damping {
        linear_damping: 2.0,
        angular_damping: 2.0,
    };

    // 🧹 Elimina jugadores anteriores
    for entity in &existing_players {
        commands.entity(entity).despawn_recursive();
    }

    // 🔵 Jugadores del jugador 1
    if let Some(f1) = formations.player1 {
        let positions = get_formation_positions(f1, true);
        for pos in positions {
            commands.spawn((
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
                PlayerDisk { player_id: 1 },
            ));
        }
    }

    // 🔴 Jugadores del jugador 2
    if let Some(f2) = formations.player2 {
        let positions = get_formation_positions(f2, false);
        for pos in positions {
            commands.spawn((
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
                PlayerDisk { player_id: 2 },
            ));
        }
    }
}

