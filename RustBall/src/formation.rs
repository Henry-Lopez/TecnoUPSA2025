//! src/formation.rs
//! --------------------------------------------------------------
//! Spawnea los discos de una formación inicial o reconstruye desde snapshot
//! --------------------------------------------------------------

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::{PlayerDisk, OwnedBy};
use crate::resources::{BackendInfo, Formation};
use crate::snapshot::FormacionData;

/// Devuelve las posiciones de una formación; se reflejan si `is_left = true`
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

/// Spawnea los discos de la formación `data`.
///
/// Se usa:
/// * Al iniciar la partida (desde `/formacion`),
/// * Cuando un snapshot trae formaciones pero aún no hay jugadas.
pub fn spawn_formation_for(
    data:          &FormacionData,
    commands:      &mut Commands,
    asset_server:  &Res<AssetServer>,
    backend_info:  &BackendInfo,
) {
    let is_left = data.id_usuario == backend_info.id_left;

    let formation = match data.formacion.as_str() {
        "1-2-1-1"   => Formation::Rombo1211,
        "2-2-1"     => Formation::Muro221,
        "1-1-3"     => Formation::Ofensiva113,
        "2-1-1-1"   => Formation::Diamante2111,
        _           => Formation::Rombo1211, // fallback
    };

    let texture = if is_left {
        asset_server.load("circulobarca.png")
    } else {
        asset_server.load("circuloparis.png")
    };

    let damping = Damping {
        linear_damping:  2.0,
        angular_damping: 2.0,
    };

    for (idx, pos) in get_formation_positions(formation, is_left)
        .into_iter()
        .enumerate()
    {
        commands.spawn((
            // ───── Render & Transform ─────
            SpriteBundle {
                texture:   texture.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 10.0),
                sprite: Sprite {
                    color:        Color::WHITE,
                    custom_size:  Some(Vec2::splat(70.0)),
                    ..default()
                },
                ..default()
            },
            // ───── Física Rapier ─────
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
            // ───── Componentes de juego ─────
            PlayerDisk {
                player_id:      data.id_usuario, // ya no se usa para lógica de turno
                id_usuario_real: data.id_usuario,
            },
            OwnedBy(data.id_usuario),            // 👈 necesario para selección de turno
            Name::new(format!("disk_{}_{}", data.id_usuario, idx)),
        ));
    }
}
