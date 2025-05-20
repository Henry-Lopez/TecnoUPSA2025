//! powerup.rs  — versión sin parpadeo

use bevy::prelude::*;
use bevy::text::{Text2dBundle, TextAlignment, TextStyle};
use bevy_rapier2d::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::components::{PlayerDisk, PowerUpLabel};

/* ───────── Config ───────── */
pub const TURN_INTERVAL_FOR_POWERUP: usize = 1;

/* ───────── Componentes ───── */
#[derive(Component)] pub struct PowerUp;
#[derive(Component)] pub struct PendingSpeedBoost;
#[derive(Component)] pub struct PendingDoubleBounce;
#[derive(Component)] pub struct PendingDoubleTurn;

#[derive(Component)] pub struct PowerUpType(pub usize);

/* ───────── Resources ─────── */
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

/* ───────── Startup ───────── */
pub fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontHandles {
        // ← usa Linebeam.ttf en lugar de FiraSans-Bold.ttf
        fira_bold: asset_server.load("fonts/Linebeam.ttf"),
    });
}
/* ───────── Spawner ───────── */
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

    let mut options = vec![("rayooo.png", 0), ("rebote.png", 1), ("dobleturno.png", 2)];
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

/* ───────── Colisiones ────── */
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
            } else { continue };

            commands.entity(pup_entity).despawn_recursive();
            control.active = false;

            // Retira tipo anterior y aplica nuevo
            commands.entity(disk).remove::<PowerUpType>();

            match pup_type {
                0 => {
                    commands.entity(disk)
                        .insert((PendingSpeedBoost, PowerUpType(0)));
                }
                1 => {
                    commands.entity(disk)
                        .insert((PendingDoubleBounce, PowerUpType(1)));
                }
                2 => {
                    commands.entity(disk)
                        .insert((PendingDoubleTurn, PowerUpType(2)));
                }
                _ => {}
            }

        }
    }
}

/* ───────── Texto encima del jugador ───────── */
pub fn attach_powerup_label_once(
    mut commands: Commands,
    fonts: Res<FontHandles>,
    query: Query<Entity, (With<PlayerDisk>, Without<PowerUpLabel>)>,
) {
    for disk in &query {
        let label = commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: fonts.fira_bold.clone(),
                        font_size: 18.0,
                        color: Color::YELLOW,
                    },
                )
                    .with_alignment(TextAlignment::Center),
                transform: Transform::from_xyz(0.0, 45.0, 1000.0),
                ..default()
            },
            PowerUpLabel,
        )).id();

        commands.entity(disk).add_child(label);
    }
}

/* ───────── Actualiza / oculta el texto ─────── */
pub fn update_powerup_labels(
    mut text_q: Query<(&mut Text, &mut Visibility), With<PowerUpLabel>>,
    parent_q: Query<(
        &Children,
        Option<&PowerUpType>,
        Option<&PendingSpeedBoost>,
        Option<&PendingDoubleBounce>,
        Option<&PendingDoubleTurn>,
    ), With<PlayerDisk>>,
) {
    for (children, typ, spd, bnc, trn) in &parent_q {
        // ¿qué power-up activo?
        let pup = if spd.is_some() { Some(0) }
        else if bnc.is_some() { Some(1) }
        else if trn.is_some() { Some(2) }
        else { typ.map(|t| t.0) };

        for &child in children {
            if let Ok((mut txt, mut vis)) = text_q.get_mut(child) {
                if let Some(kind) = pup {
                    txt.sections[0].value = match kind {
                        0 => "Velocidad",
                        1 => "Doble Rebote",
                        2 => "Doble Turno",
                        _ => "",
                    }.to_string();
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn remove_powerup_label(
    disks: Query<(
        Option<&PendingSpeedBoost>,
        Option<&PendingDoubleBounce>,
        Option<&PendingDoubleTurn>,
        Option<&PowerUpType>,
        &Children,
    ), With<PlayerDisk>>,
    mut text_q: Query<&mut Visibility, With<PowerUpLabel>>,
) {
    for (spd, bnc, trn, typ, children) in &disks {
        if spd.is_none() && bnc.is_none() && trn.is_none() && typ.is_none() {
            for &child in children {
                if let Ok(mut vis) = text_q.get_mut(child) {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}
