use bevy::prelude::*;
use bevy::ui::Interaction;
use crate::components::*;
use crate::resources::*;

pub fn update_turn_text(
    turn_state: Res<TurnState>,
    mut query: Query<&mut Text, With<TurnText>>,
) {
    if turn_state.is_changed() {
        for mut text in &mut query {
            text.sections[0].value = format!("Turno: Jugador {}", turn_state.current_turn);
        }
    }
}

pub fn update_score_text(
    scores: Res<Scores>,
    mut texts: Query<&mut Text, With<ScoreText>>,
) {
    if scores.is_changed() {
        for mut text in &mut texts {
            text.sections[0].value = format!("P1: {}  -  P2: {}", scores.left, scores.right);
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

use crate::formation_selection::SelectionButton;

pub fn animate_selection_buttons(
    turn: Res<PlayerFormations>,
    mut query: Query<(&Interaction, &SelectionButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, button, mut color) in &mut query {
        let is_selected = match button.player_id {
            1 => turn.player1 == Some(button.formation),
            2 => turn.player2 == Some(button.formation),
            _ => false,
        };

        *color = match *interaction {
            Interaction::Pressed => BackgroundColor(Color::rgb(0.2, 0.7, 0.2)), // clic actual
            Interaction::Hovered if !is_selected => BackgroundColor(Color::rgb(0.5, 0.5, 0.9)), // hover visual
            _ if is_selected => BackgroundColor(Color::rgb(0.2, 0.7, 0.2)), // selección guardada
            _ => BackgroundColor(Color::DARK_GRAY), // default
        };
    }
}

