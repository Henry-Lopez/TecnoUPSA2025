// ---------------------------------------------------------------------------
// powerup.rs  (o el nombre que uses para el módulo)
// ---------------------------------------------------------------------------
use bevy::prelude::*;
use bevy::text::{BreakLineOn, Text2dBundle, TextAlignment, TextSection, TextStyle};
use bevy_rapier2d::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::components::{PlayerDisk, PowerUpLabel};

// ────────────────────────────── Constantes ────────────────────────────────
pub const TURN_INTERVAL_FOR_POWERUP: usize = 1;

// ───────────────────────────── Etiquetas ECS ──────────────────────────────
#[derive(Component)] pub struct PowerUp;
#[derive(Component)] pub struct PendingSpeedBoost;
#[derive(Component)] pub struct PendingDoubleBounce;
#[derive(Component)] pub struct PendingDoubleTurn;

/// Guarda el tipo de power-up aplicado al disco
#[derive(Component)] pub struct PowerUpType(pub usize);

/// Control global para «cool-down» y antirepetición
#[derive(Resource, Default)]
pub struct PowerUpControl {
    pub turns_since_last: usize,
    pub active: bool,
    pub last_type: Option<usize>,
}

// ─────────────────────────── Generar power-up ─────────────────────────────
pub fn spawn_power_up_if_needed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut control: ResMut<PowerUpControl>,
) {
    if control.active || control.turns_since_last < TURN_INTERVAL_FOR_POWERUP {
        return;
    }

    let mut rng = thread_rng();
    let pos = Vec2::new(rng.gen_range(-400.0..400.0), rng.gen_range(-300.0..300.0));

    let mut options = vec![
        ("rayito.png", 0),
        ("rebote.png", 1),
        ("doble_turno.png", 2),
    ];
    if let Some(last) = control.last_type {
        options.retain(|(_, t)| *t != last);
    }
    let (tex, t) = *options.choose(&mut rng).unwrap();

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(tex),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 10.0),
            ..default()
        },
        PowerUpType(t),                // necesario para el label previo a la colisión
        PowerUp,
        Collider::ball(20.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    ));

    control.active = true;
    control.turns_since_last = 0;
    control.last_type = Some(t);
}

// ─────────────────────── Detección de colisión ────────────────────────────
pub fn detect_powerup_collision(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    disks:    Query<(Entity, &PlayerDisk)>,
    powerups: Query<(Entity, &PowerUpType), With<PowerUp>>,
    mut control: ResMut<PowerUpControl>,
) {
    for ev in collisions.read() {
        if let CollisionEvent::Started(a, b, _) = ev {
            let (disk, pup_entity, pup_type) = if let (Ok((d, _)), Ok((p, p_t))) =
                (disks.get(*a), powerups.get(*b))
            {
                (d, p, p_t.0)
            } else if let (Ok((d, _)), Ok((p, p_t))) = (disks.get(*b), powerups.get(*a)) {
                (d, p, p_t.0)
            } else {
                continue;
            };

            match pup_type {
                0 => {               // ⚡ Velocidad
                    commands.entity(disk).insert((PendingSpeedBoost, PowerUpType(0)));
                }
                1 => {               // 🎾 Doble Rebote
                    commands.entity(disk).insert((PendingDoubleBounce, PowerUpType(1)));
                }
                2 => {               // ⏩ Doble Turno
                    commands.entity(disk).insert((PendingDoubleTurn,  PowerUpType(2)));
                }
                _ => {}
            }

            commands.entity(pup_entity).despawn_recursive();
            control.active = false;
        }
    }
}


// ───────────────────────── Mostrar etiqueta ───────────────────────────────
pub fn attach_powerup_label(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Transform, &PowerUpType), (With<PlayerDisk>, Without<PowerUpLabel>)>,
) {
    for (disk, _tx, pup) in &query {
        let text = match pup.0 {
            0 => "Velocidad",
            1 => "Doble Rebote",
            2 => "Doble Turno",
            _ => "Power-Up",
        };

        let label = commands
            .spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            text,
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 16.0,
                                color: Color::YELLOW.with_a(0.85),
                            },
                        )],
                        alignment: TextAlignment::Center,
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    transform: Transform::from_xyz(0.0, 25.0, 20.0), // Z alto
                    ..default()
                },
                PowerUpLabel,
            ))
            .id();

        commands.entity(disk).add_child(label);
    }
}

// ────────────────── Limpiar etiqueta al consumir efecto ───────────────────
pub fn remove_powerup_label(
    mut commands: Commands,
    disks: Query<(
        Entity,
        Option<&PendingSpeedBoost>,
        Option<&PendingDoubleBounce>,
        Option<&PendingDoubleTurn>,
        Option<&Children>,
    ), With<PlayerDisk>>,
    labels: Query<&PowerUpLabel>,
) {
    for (disk, spd, bnc, trn, kids) in &disks {
        if spd.is_some() || bnc.is_some() || trn.is_some() {
            continue; // aún tiene power-up
        }

        if let Some(children) = kids {
            for &child in children {
                if labels.get(child).is_ok() {
                    commands.entity(child).despawn_recursive();
                }
            }
        }
        commands.entity(disk).remove::<PowerUpType>();
    }
}
