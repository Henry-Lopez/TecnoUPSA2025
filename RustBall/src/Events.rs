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


