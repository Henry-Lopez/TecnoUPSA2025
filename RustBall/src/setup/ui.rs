use bevy::prelude::*;
use crate::components::{TurnText, ScoreText, PowerBar};
use crate::resources::PowerBarBackground;

pub fn spawn_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Texto del turno
    commands.spawn((
        TextBundle::from_section(
            "Turno: Jugador 1",
            TextStyle {
                font: asset_server.load("fonts/Linebeam.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
        TurnText,
    ));

    // Texto del puntaje
    commands.spawn((
        TextBundle::from_section(
            "Jugador 1: 0 | Jugador 2: 0",
            TextStyle {
                font: asset_server.load("fonts/Linebeam.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Px(10.0),
                ..default()
            }),
        ScoreText,
    ));

    // Barra de poder
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                ..default()
            },
            PowerBarBackground,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(0.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::YELLOW),
                    ..default()
                },
                PowerBar,
            ));
        });
}

pub fn cleanup_power_bar(
    mut commands: Commands,
    query: Query<Entity, With<PowerBarBackground>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
