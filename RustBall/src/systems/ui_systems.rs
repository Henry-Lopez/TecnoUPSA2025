use bevy::prelude::*;
use bevy::ui::{BackgroundColor, Interaction};

use crate::components::*;
use crate::resources::*;
use crate::formation_selection::SelectionButton;

/* ───────────────────────── HUD dinámico ───────────────────────── */

use crate::snapshot::CurrentPlayerId;

use crate::resources::BackendInfo;

pub fn update_turn_text(
    current_player_id: Res<CurrentPlayerId>,
    backend_info: Res<BackendInfo>,
    mut query: Query<&mut Text, With<TurnText>>,
) {
    if current_player_id.is_changed() {
        for mut text in &mut query {
            if current_player_id.0 == backend_info.my_uid {
                text.sections[0].value = "🎯 Tu turno".to_string();
            } else {
                text.sections[0].value = "⏳ Turno del rival".to_string();
            }
        }
    }
}

pub fn update_score_text(
    scores: Res<Scores>,
    mut texts: Query<&mut Text, With<ScoreText>>,
) {
    if scores.is_changed() {
        for mut text in &mut texts {
            text.sections[0].value =
                format!("P1: {}  -  P2: {}", scores.left, scores.right);
        }
    }
}

pub fn update_power_bar(
    turn_state: Res<TurnState>,
    mut query: Query<&mut Style, With<PowerBar>>,
) {
    if let Some(mut style) = query.iter_mut().next() {
        style.width = Val::Px(200.0 * turn_state.power);
    }
}

/* ─────────────────── Botones de selección de formación ─────────────────── */

pub fn animate_selection_buttons(
    formations: Res<PlayerFormations>,
    mut query: Query<(&Interaction, &SelectionButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, button, mut color) in &mut query {
        /* ¿Este botón corresponde a alguna de las formaciones elegidas? */
        let is_selected =
            formations.player1 == Some(button.formation) ||
                formations.player2 == Some(button.formation);

        *color = match *interaction {
            Interaction::Pressed =>                        // clic actual
                BackgroundColor(Color::rgb(0.20, 0.70, 0.20)),
            Interaction::Hovered if !is_selected =>        // hover sobre no-seleccionado
                BackgroundColor(Color::rgb(0.50, 0.50, 0.90)),
            _ if is_selected =>                            // botón ya elegido
                BackgroundColor(Color::rgb(0.20, 0.70, 0.20)),
            _ =>                                           // estado normal
                BackgroundColor(Color::DARK_GRAY),
        };
    }
}
