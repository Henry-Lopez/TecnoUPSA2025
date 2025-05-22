//! src/systems/apply_snapshot.rs
//! --------------------------------------------------------------
//! Reconstruye todas las fichas a partir del snapshot que envía
//! el backend.  A partir de ahora cada `PlayerDisk` guarda
//! `id_usuario_real` (el UID de MySQL) y el control del turno se
//! concede usando ese UID, de modo que cada cliente sólo pueda
//! mover sus propias fichas.
//!
//!   • Se eliminó por completo el mapeo `0/1/2 → UID`.
//!   • Las texturas se cargan vía `AssetServer`, por lo que ya no
//!     aparecen “cuadrados blancos” al refrescar la página.
//! --------------------------------------------------------------

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{OwnedBy, PlayerDisk, TurnControlled},
    resources::{BackendInfo, PlayerNames},
    snapshot::BoardSnapshot,
};

/// Crea (o recrea) todas las fichas que vienen dentro de `board`.
///
/// * **existing_disks** – todas las entidades actuales con `PlayerDisk` serán
///   despawneadas antes de spawnear las nuevas.
/// * **current_turn_id** – UID real del jugador al que le toca mover; sólo la
///   primera ficha de ese jugador recibe `TurnControlled`.
pub fn apply_board_snapshot(
    board: BoardSnapshot,
    commands: &mut Commands,
    backend_info: BackendInfo,
    existing_disks: Query<Entity, With<PlayerDisk>>,
    current_turn_id: i32,
    names: Option<PlayerNames>,
    asset_server: &Res<AssetServer>,
) {
    /* ─── 1. Limpiar fichas anteriores ─────────────────────────────── */
    for entity in existing_disks.iter() {
        commands.entity(entity).despawn_recursive();
    }

    /* ─── 2. Recursos comunes (texturas + damping) ─────────────────── */
    let tex_left  = asset_server.load("circulobarca.png");
    let tex_right = asset_server.load("circuloparis.png");

    let damping = Damping {
        linear_damping: 2.0,
        angular_damping: 2.0,
    };

    /* ─── 3. Spawnear cada pieza ────────────────────────────────────── */
    let my_uid          = backend_info.my_uid;
    let mut control_set = false; // sólo una ficha recibe TurnControlled

    for pieza in board.piezas {
        let uid_real = pieza.id_usuario_real;

        /* ¿Es jugador izquierdo o derecho? */
        let is_left   = uid_real == backend_info.id_left;
        let texture   = if is_left { tex_left.clone() } else { tex_right.clone() };

        let name_log = match &names {
            Some(n) if is_left => &n.left_name,
            Some(n)            => &n.right_name,
            None               => "desconocido",
        };

        info!("🧩 Spawn ficha UID {uid_real} – jugador {name_log}");

        /* — Sprite + cuerpo físico — */
        let mut ecmd = commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(pieza.x, pieza.y, 10.0),
                sprite: Sprite {
                    color: Color::WHITE,
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
            /* El `player_id` local (1 izquierda, 2 derecha) sólo se usa
               para colorear la UI; el UID real se guarda aparte        */
            PlayerDisk {
                player_id: if is_left { 1 } else { 2 },
                id_usuario_real: uid_real,
            },
            OwnedBy(uid_real),
            Name::new(format!("disk_user_{uid_real}")),
        ));

        /* — Dar control a la primera ficha de mi turno — */
        if uid_real == my_uid && uid_real == current_turn_id && !control_set {
            ecmd.insert(TurnControlled);
            control_set = true;
        }
    }
}
