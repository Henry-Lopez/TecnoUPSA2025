use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::components::PlayerDisk;
use crate::resources::{TurnState, EventControl};
use crate::events::RandomEvent;

#[derive(Component)]
pub struct SlipperyZone;
#[derive(Component)]
pub struct SlowZone;
#[derive(Component)]
pub struct BouncePad;
#[derive(Component)]
pub struct ActiveEffectText;

#[derive(Component)]
pub struct ZoneLifetime {
    pub turns_remaining: u8,
    pub last_turn_owner: u8,
}

// === SPAWNS ===

pub fn spawn_slippery_zone(commands: &mut Commands, position: Vec2, size: Vec2, current_turn: u8) {
    commands.spawn((
        SlipperyZone,
        ZoneLifetime {
            turns_remaining: 2,
            last_turn_owner: current_turn,
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::CYAN.with_a(0.4),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..default()
        },
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Friction::coefficient(0.01),
        Sensor,
    ));
}

pub fn spawn_slow_zone(commands: &mut Commands, position: Vec2, size: Vec2, current_turn: u8) {
    commands.spawn((
        SlowZone,
        ZoneLifetime {
            turns_remaining: 2,
            last_turn_owner: current_turn,
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED.with_a(0.4),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..default()
        },
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Friction::coefficient(1.5),
        Sensor,
    ));
}

pub fn spawn_bounce_pad(commands: &mut Commands, position: Vec2, size: Vec2, current_turn: u8) {
    commands.spawn((
        BouncePad,
        ZoneLifetime {
            turns_remaining: 2,
            last_turn_owner: current_turn,
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::ORANGE_RED.with_a(0.5),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..default()
        },
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Restitution::coefficient(2.5),
        Sensor,
    ));
}

// === LIFETIME ===

pub fn update_zone_lifetime(
    mut commands: Commands,
    mut zones: Query<(Entity, &mut ZoneLifetime)>,
    mut control: ResMut<EventControl>,
    turn_state: Res<TurnState>,
) {
    if !turn_state.is_changed() || turn_state.in_motion {
        return;
    }

    let mut despawned_any = false;
    let current = turn_state.current_turn_id as u8; // ✅ corregido

    for (entity, mut lifetime) in &mut zones {
        if current != lifetime.last_turn_owner {
            lifetime.turns_remaining -= 1;
            lifetime.last_turn_owner = current;

            if lifetime.turns_remaining == 0 {
                commands.entity(entity).despawn_recursive();
                despawned_any = true;
            }
        }
    }

    if despawned_any && zones.iter().count() == 1 {
        control.event_active = false;
        control.current_event = None;
        control.turns_since_last = 0;
    }
}

// === EFECTOS ===

pub fn apply_zone_effects(
    mut disks: Query<(&Transform, &mut Velocity), With<PlayerDisk>>,
    zones: Query<(&Transform, &Sprite, Option<&SlipperyZone>, Option<&SlowZone>)>,
) {
    for (disk_tf, mut velocity) in &mut disks {
        let disk_pos = disk_tf.translation.truncate();

        for (zone_tf, sprite, is_slippery, is_slow) in &zones {
            if let Some(size) = sprite.custom_size {
                let zone_pos = zone_tf.translation.truncate();
                let half = size / 2.0;
                let inside = (disk_pos.x >= zone_pos.x - half.x && disk_pos.x <= zone_pos.x + half.x)
                    && (disk_pos.y >= zone_pos.y - half.y && disk_pos.y <= zone_pos.y + half.y);

                if inside {
                    if is_slippery.is_some() {
                        velocity.linvel *= 1.1;
                    } else if is_slow.is_some() {
                        velocity.linvel *= 0.8;
                    }
                }
            }
        }
    }
}

// === TEXTO ===

pub fn update_active_effect_text(
    mut commands: Commands,
    control: Res<EventControl>,
    mut query: Query<(Entity, &mut Text), With<ActiveEffectText>>,
    asset_server: Res<AssetServer>,
) {
    let mensaje = match control.current_event {
        Some(RandomEvent::SlipperyZone) => "Efecto actual: Zona resbalosa",
        Some(RandomEvent::SlowZone) => "Efecto actual: Zona lenta",
        Some(RandomEvent::BouncePad) => "Efecto actual: Trampolín",
        None => "",
    };

    if let Some((_, mut text)) = query.iter_mut().next() {
        text.sections[0].value = mensaje.to_string();
    } else if !mensaje.is_empty() {
        commands.spawn((
            TextBundle::from_section(
                mensaje,
                TextStyle {
                    font: asset_server.load("fonts/Linebeam.ttf"),
                    font_size: 28.0,
                    color: Color::ORANGE,
                },
            )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                }),
            ActiveEffectText,
        ));
    }
}

pub fn hide_effect_text_if_none(
    mut commands: Commands,
    query: Query<Entity, With<ActiveEffectText>>,
    control: Res<EventControl>,
) {
    if control.current_event.is_none() {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// === CLEANUP MANUAL ===

pub fn cleanup_zones(
    mut commands: Commands,
    query: Query<Entity, Or<(With<SlipperyZone>, With<SlowZone>, With<BouncePad>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
