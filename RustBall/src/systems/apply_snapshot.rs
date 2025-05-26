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

/// Crea / recrea todas las piezas que llegan en `board`.
///
/// * **existing_disks**  – despawnea todas las entidades con `PlayerDisk`.
/// * **existing_ball**   – despawnea la pelota si ya existía.
/// * **current_turn_id** – UID real del jugador al que le toca mover; sólo
///                         la primera ficha de ese jugador recibe
///                         `TurnControlled`.
pub fn apply_board_snapshot(
    board: BoardSnapshot,
    commands: &mut Commands,
    backend_info: BackendInfo,
    existing_disks: Query<Entity, With<PlayerDisk>>,
    existing_ball : Query<Entity, With<Ball>>,     // ← nuevo
    current_turn_id: i32,
    names: Option<PlayerNames>,
    asset_server: &Res<AssetServer>,
) {
    /* ─── 1. Limpiar fichas y pelota anteriores ───────────────────── */
    for e in &existing_disks {
        commands.entity(e).despawn_recursive();
    }
    for entity in existing_ball.iter() {
        commands.entity(entity).despawn_recursive();
    }

    /* ─── 2. Recursos comunes (texturas + damping) ────────────────── */
    let tex_left  = asset_server.load("circulobarca.png");
    let tex_right = asset_server.load("circuloparis.png");
    let tex_ball  = asset_server.load("pelota.png");

    let damping = Damping {
        linear_damping : 2.0,
        angular_damping: 2.0,
    };

    /* ─── 3. Spawnear cada pieza ──────────────────────────────────── */
    let my_uid          = backend_info.my_uid;
    let mut control_set = false;          // sólo 1 ficha con TurnControlled

    for PiezaPos { id, x, y, id_usuario_real } in board.piezas {
        /* ───── Pelota ───── */
        if id == -1 {
            commands.spawn((
                SpriteBundle {
                    texture   : tex_ball.clone(),
                    transform : Transform::from_xyz(x, y, 12.0),
                    sprite    : Sprite {
                        custom_size: Some(Vec2::splat(48.0)),
                        ..default()
                    },
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
            continue;
        }

        /* ───── Ficha de jugador ───── */
        let uid_real = id_usuario_real;
        let is_left  = uid_real == backend_info.id_left;

        let texture  = if is_left { tex_left.clone() } else { tex_right.clone() };
        let name_log = match &names {
            Some(n) if is_left => &n.left_name,
            Some(n)            => &n.right_name,
            None               => "desconocido",
        };
        info!("🧩 Spawn ficha UID {uid_real} – jugador {name_log}");

        let mut ecmd = commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 10.0),
                sprite: Sprite {
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
                player_id      : if is_left { 1 } else { 2 },
                id_usuario_real: uid_real,
            },
            OwnedBy(uid_real),
            Name::new(format!("disk_user_{uid_real}")),
        ));

        // Dar control a la primera ficha del jugador en turno
        if uid_real == my_uid && uid_real == current_turn_id && !control_set {
            ecmd.insert(TurnControlled);
            control_set = true;
        }
    }
}
