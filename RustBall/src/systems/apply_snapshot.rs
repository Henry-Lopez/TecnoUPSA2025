//! src/systems/apply_snapshot.rs
//! --------------------------------------------------------------
//! Reconstruye el tablero a partir del snapshot del backend:
//!
//!   • Fichas  → PlayerDisk + OwnedBy
//!   • Pelota  → Ball   (id == -1)
//!
//! Cada PlayerDisk conserva `id_usuario_real`, de forma que cada
//! cliente sólo puede mover las suyas.
//! --------------------------------------------------------------

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{Ball, OwnedBy, PlayerDisk, TurnControlled},
    resources::{BackendInfo, PlayerNames},
    snapshot::{BoardSnapshot, PiezaPos},
};

/// Reconstruye todas las fichas y la pelota según el snapshot recibido.
///
/// - **existing_disks** – despawnea todas las entidades con `PlayerDisk`.
/// - **existing_ball**  – mantiene o spawnea la pelota (no duplica).
/// - **current_turn_id** – UID real del jugador al que le toca mover; sólo
///                         la primera ficha de ese jugador recibe
///                         `TurnControlled`.
pub fn apply_board_snapshot(
    board: BoardSnapshot,
    commands: &mut Commands,
    backend_info: BackendInfo,
    existing_disks: Query<Entity, With<PlayerDisk>>,
    existing_ball: Query<(Entity, &Transform), With<Ball>>,
    current_turn_id: i32,
    names: Option<PlayerNames>,
    asset_server: &Res<AssetServer>,
) {
    // 1. Despawner fichas anteriores:
    for e in &existing_disks {
        commands.entity(e).despawn_recursive();
    }

    // 2. Cargar texturas y propiedades comunes:
    let tex_left  = asset_server.load("circulobarca.png");
    let tex_right = asset_server.load("circuloparis.png");
    let tex_ball  = asset_server.load("pelota.png");
    let damping = Damping { linear_damping: 2.0, angular_damping: 2.0 };

    // 3. Procesar cada pieza del snapshot:
    let my_uid = backend_info.my_uid;
    let mut control_set = false;
    let mut ball_spawned = false;

    for PiezaPos { id, x, y, id_usuario_real } in board.piezas {
        if id == -1 {
            // Pelota: si ya existe, actualizar posición; si no, spawnear
            if let Ok((entity, _)) = existing_ball.get_single() {
                commands.entity(entity)
                    .insert(Transform::from_xyz(x, y, 12.0));
            } else {
                commands.spawn((
                    SpriteBundle {
                        texture: tex_ball.clone(),
                        transform: Transform::from_xyz(x, y, 12.0),
                        sprite: Sprite { custom_size: Some(Vec2::splat(48.0)), ..default() },
                        ..default()
                    },
                    RigidBody::Dynamic,
                    Collider::ball(20.0),
                    Restitution::coefficient(1.0),
                    ActiveEvents::COLLISION_EVENTS,
                    Velocity::zero(),
                    damping.clone(),
                    LockedAxes::ROTATION_LOCKED,
                    Sleeping::disabled(),
                    Ball,
                    Name::new("ball"),
                ));
            }
            ball_spawned = true;
            continue;
        }

        // Ficha de jugador:
        let is_left = id_usuario_real == backend_info.id_left;
        let texture = if is_left { tex_left.clone() } else { tex_right.clone() };
        let name_log = match &names {
            Some(n) if is_left => &n.left_name,
            Some(n)            => &n.right_name,
            None               => "desconocido",
        };
        info!("🧩 Spawn ficha UID {} – jugador {}", id_usuario_real, name_log);

        let mut ecmd = commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 10.0),
                sprite: Sprite { custom_size: Some(Vec2::splat(70.0)), ..default() },
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
            PlayerDisk { player_id: if is_left { 1 } else { 2 }, id_usuario_real },
            OwnedBy(id_usuario_real),
            Name::new(format!("disk_user_{}", id_usuario_real)),
        ));

        // Asignar control a la primera ficha del jugador en turno:
        if !control_set && id_usuario_real == my_uid && id_usuario_real == current_turn_id {
            ecmd.insert(TurnControlled);
            control_set = true;
        }
    }

    // 4. Si el snapshot no incluye pelota, despawnear si existe:
    if !ball_spawned {
        for (entity, _) in existing_ball.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
