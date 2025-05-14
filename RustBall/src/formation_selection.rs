use bevy::prelude::*;
use crate::resources::{Formation, PlayerFormations, AppState};
use crate::components::FormationMenu;

#[derive(Component)]
pub struct SelectionButton {
    pub player_id: u8,
    pub formation: Formation,
}

pub fn show_formation_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/Linebeam.ttf");

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..default()
    });

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        },
        FormationMenu,
    ))
        .with_children(|parent| {
            for (i, player) in ["Jugador 1", "Jugador 2"].iter().enumerate() {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        format!("{player}, elige tu formacion"),
                        TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 0.0, 10.0),
                    ..default()
                });

                for &formation in &[
                    Formation::Rombo1211,
                    Formation::Muro221,
                    Formation::Ofensiva113,
                    Formation::Diamante2111,
                ] {
                    parent.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.0),
                                height: Val::Px(40.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            transform: Transform::from_xyz(0.0, 0.0, 5.0),
                            ..default()
                        },
                        SelectionButton {
                            player_id: (i + 1) as u8,
                            formation,
                        },
                    ))
                        .with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                format!("{:?}", formation),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                }
            }
        });
}

pub fn handle_formation_click(
    mut interaction_query: Query<(&Interaction, &SelectionButton), (Changed<Interaction>, With<Button>)>,
    mut formations: ResMut<PlayerFormations>,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    for (interaction, button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match button.player_id {
                1 => formations.player1 = Some(button.formation),
                2 => formations.player2 = Some(button.formation),
                _ => {}
            }
        }
    }

    if formations.player1.is_some() && formations.player2.is_some() {
        match current_state.get() {
            AppState::FormationSelection => next_state.set(AppState::InGame),
            AppState::FormationChange => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}

pub fn cleanup_formation_ui(
    mut commands: Commands,
    query: Query<Entity, With<FormationMenu>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
