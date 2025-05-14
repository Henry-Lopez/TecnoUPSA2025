 use bevy::prelude::*;

 #[derive(Component)]
 pub struct PlayerDisk {
     pub player_id: usize,
 }

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct TurnControlled;

#[derive(Component)]
pub struct GoalZone {
    pub is_left: bool,
}

#[derive(Component)]
pub struct TurnText;

 #[derive(Component)]
 pub struct ScoreText;

 #[derive(Component)]
 pub struct PowerBar;

 #[derive(Component)]
 pub struct FormationMenu;


