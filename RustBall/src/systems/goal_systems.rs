use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::setup::camera::GameCamera;

#[derive(Component)]
pub struct GoalBanner;

#[derive(Resource)]
pub struct GoalBannerTimer {
    pub timer: Timer,
}

pub fn detect_goal(
    mut collision_events: EventReader<CollisionEvent>,
    goals: Query<(&GoalZone, Entity)>,
    balls: Query<Entity, With<Ball>>,
    mut goal_events: EventWriter<GoalEvent>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(a, b, _) = event {
            for (goal, goal_entity) in &goals {
                for ball_entity in &balls {
                    if (*a == goal_entity && *b == ball_entity)
                        || (*b == goal_entity && *a == ball_entity)
                    {
                        goal_events.send(GoalEvent {
                            scored_by_left: !goal.is_left,
                        });
                    }
                }
            }
        }
    }
}

pub fn handle_goal(
    mut goal_events: EventReader<GoalEvent>,
    mut scores: ResMut<Scores>,
    mut turn_state: ResMut<TurnState>,
    mut sprites: Query<&mut Sprite>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
    backend_info: Res<BackendInfo>,
) {
    for event in goal_events.read() {
        // 🎯 Actualizar marcador
        if event.scored_by_left {
            scores.left += 1;
            println!("Gol para el jugador izquierdo! Puntos: {}", scores.left);
        } else {
            scores.right += 1;
            println!("Gol para el jugador derecho! Puntos: {}", scores.right);
        }

        // 🔄 Limpiar estado actual del turno
        if let Some(entity) = turn_state.selected_entity {
            if let Ok(mut sprite) = sprites.get_mut(entity) {
                sprite.color = Color::WHITE;
            }
            commands.entity(entity).remove::<TurnControlled>();
        }

        turn_state.in_motion = false;
        turn_state.selected_entity = None;
        turn_state.aim_direction = Vec2::ZERO;
        turn_state.power = 0.0;

        // 🔁 Cambiar al otro jugador (basado en ID reales)
        turn_state.current_turn_id = if turn_state.current_turn_id == backend_info.id_left {
            backend_info.id_right
        } else {
            backend_info.id_left
        };

        // ✨ Mostrar banner de gol
        commands.spawn((
            TextBundle {
                text: Text::from_section(
                    "¡GOOOOL!",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 80.0,
                        color: Color::GOLD,
                    },
                )
                    .with_alignment(TextAlignment::Center),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(40.0),
                    left: Val::Percent(35.0),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 100.0),
                ..default()
            },
            GoalBanner,
        ));

        // ⏱️ Guardamos el siguiente estado dependiendo del marcador
        if scores.left == 3 || scores.right == 3 {
            next_state.set(AppState::GameOver);
        } else {
            next_state.set(AppState::GoalScored);
        }
    }
}

pub fn setup_goal_timer(mut commands: Commands) {
    commands.insert_resource(GoalBannerTimer {
        timer: Timer::from_seconds(4.0, TimerMode::Once),
    });
}

pub fn goal_banner_fadeout(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<GoalBannerTimer>,
    mut query: Query<(Entity, &mut Text, &mut Transform), With<GoalBanner>>,
) {
    timer.timer.tick(time.delta());

    for (entity, mut text, mut transform) in &mut query {
        let progress = timer.timer.elapsed_secs() / timer.timer.duration().as_secs_f32();
        let alpha = 1.0 - progress;
        let scale = 1.0 + (0.05 * (progress * std::f32::consts::PI * 4.0).sin());

        text.sections[0].style.color.set_a(alpha.clamp(0.0, 1.0));
        transform.scale = Vec3::splat(scale);

        if timer.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn wait_and_change_state(
    mut next_state: ResMut<NextState<AppState>>,
    timer: Res<GoalBannerTimer>,
    scores: Res<Scores>,
) {
    if timer.timer.finished() {
        if scores.left >= 3 || scores.right >= 3 {
            next_state.set(AppState::GameOver);
        } else {
            next_state.set(AppState::FormationChange);
        }
    }
}

pub fn despawn_game_entities(
    mut commands: Commands,
    players: Query<Entity, With<PlayerDisk>>,
    balls: Query<Entity, With<Ball>>,
    goals: Query<Entity, With<GoalZone>>,
    fixed_bodies: Query<Entity, (With<RigidBody>, Without<PlayerDisk>, Without<Ball>)>,
    cameras: Query<Entity, With<GameCamera>>,
) {
    for entity in players.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in balls.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in goals.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in fixed_bodies.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in cameras.iter() {
        commands.entity(entity).despawn_recursive();
    }

    println!("✅ Entidades físicas y visuales eliminadas tras GameOver.");
}
