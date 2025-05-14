use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::components::*;

pub fn spawn_goals(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // 📏 Dimensiones
    let goal_height = 200.0;
    let goal_width = 100.0;
    let wall_thickness = 10.0;
    let field_width = 1100.0;
    let half_field = field_width / 2.0;
    let half_w = goal_width / 2.0;
    let half_h = goal_height / 2.0;
    let z_sensor = 0.0;
    let z_struct = 0.1;

    let x_izq = -half_field - 10.0;
    let x_der = half_field + 10.0;

    // ================= IZQUIERDO =================
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("arcoizq.png"),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(goal_width, goal_height)),
                ..default()
            },
            transform: Transform::from_xyz(x_izq, 0.0, z_sensor),
            ..default()
        },
        Collider::cuboid(half_w - 35.0, half_h - 70.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        GoalZone { is_left: true },
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(5.0, goal_height)),
                ..default()
            },
            transform: Transform::from_xyz(x_izq + 15.0 - half_w, 0.0, z_struct + 0.01),
            ..default()
        },
        Collider::cuboid(2.5, half_h),
        RigidBody::Fixed,
        Restitution::coefficient(6.5),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(goal_width, wall_thickness)),
                ..default()
            },
            transform: Transform::from_xyz(x_izq - 35.0, half_h, z_struct),
            ..default()
        },
        Collider::cuboid(half_w, wall_thickness / 2.0),
        RigidBody::Fixed,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(goal_width, wall_thickness)),
                ..default()
            },
            transform: Transform::from_xyz(x_izq - 35.0, -half_h, z_struct),
            ..default()
        },
        Collider::cuboid(half_w, wall_thickness / 2.0),
        RigidBody::Fixed,
    ));

    // ================= DERECHO =================
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("arcoder.png"),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(goal_width, goal_height)),
                ..default()
            },
            transform: Transform::from_xyz(x_der, 0.0, z_sensor),
            ..default()
        },
        Collider::cuboid(half_w - 35.0, half_h - 70.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        GoalZone { is_left: false },
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(20.0, goal_height)),
                ..default()
            },
            transform: Transform::from_xyz(x_der - 15.0 + half_w, 0.0, z_struct + 0.01),
            ..default()
        },
        Collider::cuboid(wall_thickness / 2.0, half_h),
        RigidBody::Fixed,
        Restitution::coefficient(6.5),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(goal_width, wall_thickness)),
                ..default()
            },
            transform: Transform::from_xyz(x_der + 35.0, half_h, z_struct),
            ..default()
        },
        Collider::cuboid(half_w, wall_thickness / 2.0),
        RigidBody::Fixed,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(goal_width, wall_thickness)),
                ..default()
            },
            transform: Transform::from_xyz(x_der + 35.0, -half_h, z_struct),
            ..default()
        },
        Collider::cuboid(half_w, wall_thickness / 2.0),
        RigidBody::Fixed,
    ));
}
