use bevy::prelude::*;

#[derive(Event)]
pub struct GoalEvent {
    pub scored_by_left: bool,
}

#[derive(Clone, Debug)]
pub enum RandomEvent {
    SlipperyZone,
    SlowZone,
    BouncePad,
}

#[derive(Event)]
pub struct FormationChosenEvent {
    pub formacion: String,   // "1-2-1-1", etc.
    pub turno_inicio: i32,
}

#[derive(Event)]
pub struct LocalTurnFinishedEvent;   // disparado por el cliente cuando termina su movimiento

#[derive(Event)]
pub struct RemoteTurnArrivedEvent;   // disparado por el polling / websocket al recibir turno rival

