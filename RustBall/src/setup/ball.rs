use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::components::Ball;

pub fn spawn_ball(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let damping = Damping {
        linear_damping: 2.0,
        angular_damping: 2.0,
    };

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("pelota.png"),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(20.0),
        Restitution::coefficient(1.0),
        ActiveEvents::COLLISION_EVENTS,
        ExternalImpulse::default(),
        ExternalForce::default(),
        AdditionalMassProperties::Mass(1.0),
        Velocity::zero(),
        damping,
        LockedAxes::ROTATION_LOCKED,
        Sleeping::disabled(),
        Ball,
    ));
}
