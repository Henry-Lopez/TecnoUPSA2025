use bevy::prelude::*;
use crate::resources::{EventControl, TurnState};
use crate::events::RandomEvent;
use crate::zone::{spawn_slippery_zone, spawn_slow_zone, spawn_bounce_pad};
use rand::{thread_rng, Rng}; // ✅ Usamos rand aquí

pub const EVENT_INTERVAL_TURNS: usize = 3;

pub fn trigger_random_event_system(
    mut control: ResMut<EventControl>,
    mut commands: Commands,
    turn_state: Res<TurnState>,
) {
    if control.event_active || control.turns_since_last < EVENT_INTERVAL_TURNS {
        return;
    }

    control.turns_since_last = 0;
    let current_turn: u8 = turn_state.current_turn as u8;
    let mut rng = thread_rng();

    // Posición aleatoria en el campo de juego
    let x = rng.gen_range(-300.0..=300.0);
    let y = rng.gen_range(-150.0..=150.0);
    let pos = Vec2::new(x, y);

    // Tamaño aleatorio de la zona
    let size = match rng.gen_range(0..3) {
        0 => Vec2::new(120.0, 60.0),
        1 => Vec2::new(100.0, 50.0),
        _ => Vec2::new(80.0, 80.0),
    };

    // Evitar repetir el mismo evento que el anterior
    let mut event_id = rng.gen_range(0..3);
    if let Some(ref prev) = control.current_event {
        let prev_id = match prev {
            RandomEvent::SlipperyZone => 0,
            RandomEvent::SlowZone => 1,
            RandomEvent::BouncePad => 2,
        };
        while event_id == prev_id {
            event_id = rng.gen_range(0..3);
        }
    }

    // Ejecutar evento
    let event = match event_id {
        0 => {
            spawn_slippery_zone(&mut commands, pos, size, current_turn);
            RandomEvent::SlipperyZone
        }
        1 => {
            spawn_slow_zone(&mut commands, pos, size, current_turn);
            RandomEvent::SlowZone
        }
        _ => {
            spawn_bounce_pad(&mut commands, pos, size, current_turn);
            RandomEvent::BouncePad
        }
    };

    control.current_event = Some(event);
    control.event_active = true;
}
