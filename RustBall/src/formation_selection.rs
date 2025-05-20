use bevy::prelude::*;

use crate::{
    components::FormationMenu,
    events::FormationChosenEvent,
    resources::{Formation, PlayerFormations},
};

/* ──────────────── UI ──────────────── */

#[derive(Component)]
pub struct SelectionButton {
    pub formation: Formation,
}

pub fn show_formation_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/Linebeam.ttf");

    // cámara 2D
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..default()
    });

    // contenedor vertical centrado
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
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
            // Título que cambia a “esperando…”
            parent.spawn(TextBundle::from_section(
                "Elige tu formación",
                TextStyle {
                    font: font.clone(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ));

            // Botones de formación
            for &formation in &[
                Formation::Rombo1211,
                Formation::Muro221,
                Formation::Ofensiva113,
                Formation::Diamante2111,
            ] {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(220.0),
                                height: Val::Px(42.0),
                                margin: UiRect::all(Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        },
                        SelectionButton { formation },
                    ))
                    .with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            format!("{formation:?}"),
                            TextStyle {
                                font: font.clone(),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            }
        });
}

/* ──────────────── lógica ──────────────── */

pub fn handle_formation_click(
    mut interaction_q: Query<
        (&Interaction, &SelectionButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut formations: ResMut<PlayerFormations>,
    mut ev_form_send: EventWriter<FormationChosenEvent>,
    mut menu_text_q: Query<&mut Text, (With<FormationMenu>, Without<Button>)>,
) {
    for (interaction, button, mut bg) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            // 1) Guardamos la formación local
            formations.player1 = Some(button.formation);

            // 2) Disparamos evento para que otro sistema (send_formacion_to_backend) lo envíe
            ev_form_send.send(FormationChosenEvent {
                formacion: button.formation.as_str().into(),
                turno_inicio: 0,
            });

            // 3) Feedback visual
            *bg = Color::GRAY.into();
            for mut txt in &mut menu_text_q {
                txt.sections[0].value = "⏳ Esperando a tu rival…".into();
            }
        }
    }

    // 🚫 Eliminado: NO hacemos transición a AppState::InGame aquí.
    // Eso lo controla snapshot_apply_system con proximo_turno != 0.
}

/* ──────────────── limpieza ──────────────── */

pub fn cleanup_formation_ui(mut commands: Commands, q: Query<Entity, With<FormationMenu>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
