use bevy::prelude::*;
use bevy::text::{Text2dBundle, TextAlignment, TextStyle};
use bevy_rapier2d::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::components::{PlayerDisk, PowerUpLabel, PowerUpLabelBlink};

pub const TURN_INTERVAL_FOR_POWERUP: usize = 1;

#[derive(Component)] pub struct PowerUp;
#[derive(Component)] pub struct PendingSpeedBoost;
#[derive(Component)] pub struct PendingDoubleBounce;
#[derive(Component)] pub struct PendingDoubleTurn;

#[derive(Component)] pub struct PowerUpType(pub usize);

#[derive(Resource, Default)]
pub struct PowerUpControl {
    pub turns_since_last: usize,
    pub active: bool,
    pub last_type: Option<usize>,
}

#[derive(Resource)]
pub struct FontHandles {
    pub fira_bold: Handle<Font>,
}

pub fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontHandles {
        fira_bold: asset_server.load("fonts/FiraSans-Bold.ttf"),
    });
}

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
        ("rayooo.png", 0),
        ("rebote.png", 1),
        ("dobleturno.png", 2),
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
        PowerUpType(t),
        PowerUp,
        Collider::ball(20.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    ));

    control.active = true;
    control.turns_since_last = 0;
    control.last_type = Some(t);
}

pub fn detect_powerup_collision(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    disks: Query<(Entity, &PlayerDisk)>,
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

            commands.entity(pup_entity).despawn_recursive();
            control.active = false;

            match pup_type {
                0 => {
                    commands.entity(disk).insert((PendingSpeedBoost, PowerUpType(0)));
                }
                1 => {
                    commands.entity(disk).insert((PendingDoubleBounce, PowerUpType(1)));
                }
                2 => {
                    commands.entity(disk).insert((PendingDoubleTurn, PowerUpType(2)));
                }
                _ => {}
            }

        }
    }
}

pub fn attach_powerup_label_once(
    mut commands: Commands,
    fonts: Res<FontHandles>,
    query: Query<Entity, (With<PlayerDisk>, Without<PowerUpLabel>)>,
) {
    for disk in &query {
        let label = commands
            .spawn((
                Text2dBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: fonts.fira_bold.clone(),
                            font_size: 16.0,
                            color: Color::rgba(0.8, 0.7, 0.2, 0.9),
                        },
                    )
                        .with_alignment(TextAlignment::Center),
                    transform: Transform::from_xyz(0.0, 40.0, 20.0),
                    ..default()
                },
                PowerUpLabel,
                PowerUpLabelBlink {
                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                },
                Visibility::Hidden,
            ))
            .id();

        commands.entity(disk).add_child(label);
    }
}

pub fn update_powerup_labels(
    mut text_query: Query<(&mut Text, &mut Visibility), With<PowerUpLabel>>,
    parent_query: Query<(&Children, &PowerUpType), With<PlayerDisk>>,
) {
    for (children, pup_type) in &parent_query {
        for &child in children {
            if let Ok((mut text, mut vis)) = text_query.get_mut(child) {
                let label_text = match pup_type.0 {
                    0 => "Velocidad",
                    1 => "Doble Rebote",
                    2 => "Doble Turno",
                    _ => "",
                };

                text.sections[0].value = label_text.to_string();
                *vis = Visibility::Visible;
            }
        }
    }
}

pub fn blink_powerup_labels(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut PowerUpLabelBlink)>,
) {
    for (mut text, mut blink) in &mut query {
        blink.timer.tick(time.delta());

        let phase = (blink.timer.elapsed_secs() * std::f32::consts::PI * 2.0).sin();
        let alpha = 0.3 + 0.7 * phase.abs();

        if let Some(section) = text.sections.get_mut(0) {
            let mut color = section.style.color;
            color.set_a(alpha);
            section.style.color = color;
        }
    }
}

pub fn remove_powerup_label(
    disks: Query<(
        Option<&PendingSpeedBoost>,
        Option<&PendingDoubleBounce>,
        Option<&PendingDoubleTurn>,
        &Children,
    ), With<PlayerDisk>>,
    mut text_query: Query<&mut Visibility, With<PowerUpLabel>>,
) {
    for (spd, bnc, trn, children) in &disks {
        if spd.is_none() && bnc.is_none() && trn.is_none() {
            for &child in children {
                if let Ok(mut vis) = text_query.get_mut(child) {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}
