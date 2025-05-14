use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::components::PlayerDisk;

// Cada cuántos turnos aparece un nuevo power-up
pub const TURN_INTERVAL_FOR_POWERUP: usize = 1;
#[derive(Component)]
pub struct PendingDoubleTurn;

#[derive(Component)]

pub struct PowerUp;

#[derive(Component)]
pub struct PendingSpeedBoost;

#[derive(Component)]
pub struct PendingDoubleBounce;

#[derive(Resource, Default)]
pub struct PowerUpControl {
    pub turns_since_last: usize,
    pub active: bool,
}

#[derive(Component)]
pub struct PowerUpType(pub usize);

/// Genera un power-up si es momento de hacerlo
pub fn spawn_power_up_if_needed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut control: ResMut<PowerUpControl>,
) {
    println!("⏱️ turns_since_last = {}, active = {}", control.turns_since_last, control.active);

    if control.active || control.turns_since_last < TURN_INTERVAL_FOR_POWERUP {
        return;
    }

    let pos = Vec2::new(
        fastrand::f32() * 800.0 - 400.0,
        fastrand::f32() * 600.0 - 300.0,
    );

    let (texture_path, powerup_type): (&str, usize) = match fastrand::usize(..3) {
        0 => ("rayito.png", 0),
        1 => ("rebote.png", 1),
        2 => ("doble_turno.png", 2),
        _ => unreachable!(),
    };

    println!("🟡 Power-Up generado: tipo {} en posición ({:.2}, {:.2})", powerup_type, pos.x, pos.y);

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(texture_path),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 10.0),
            ..default()
        },
        PowerUpType(powerup_type),
        PowerUp,
        Collider::ball(20.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    ));

    control.active = true;
    control.turns_since_last = 0;
}

/// Detecta si un disco recoge el power-up
pub fn detect_powerup_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    disks: Query<(Entity, &PlayerDisk)>,
    powerups: Query<(Entity, &PowerUpType), With<PowerUp>>,
    mut control: ResMut<PowerUpControl>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (disk, powerup_entity, ptype) =
                if let (Ok((disk_ent, _)), Ok((p_ent, p_t))) = (disks.get(*e1), powerups.get(*e2)) {
                    (disk_ent, p_ent, p_t.0)
                } else if let (Ok((disk_ent, _)), Ok((p_ent, p_t))) = (disks.get(*e2), powerups.get(*e1)) {
                    (disk_ent, p_ent, p_t.0)
                } else {
                    continue;
                };

            match ptype {
                0 => {
                    println!("⚡ SpeedBoost");
                    commands.entity(disk).insert(PendingSpeedBoost);
                }
                1 => {
                    println!("🎾 DoubleBounce");
                    commands.entity(disk).insert(PendingDoubleBounce);
                }
                2 => {
                    println!("🔁 DoubleTurn");
                    commands.entity(disk).insert(PendingDoubleTurn);
                }
                _ => {}
            }

            commands.entity(powerup_entity).despawn_recursive();
            control.active = false;
        }
    }
}
