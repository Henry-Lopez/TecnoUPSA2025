use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn spawn_walls(commands: &mut Commands) {
    let wall_thickness = 10.0;
    let bounds = Vec2::new(1100.0, 741.0);
    let half_w = bounds.x / 2.0;
    let half_h = bounds.y / 2.0;
    let goal_gap = 200.0; // misma altura que los arcos
    let side_wall_height = (bounds.y - goal_gap) / 2.0;

    // ⬅️ Pared izquierda arriba
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(wall_thickness, side_wall_height)),
            ..default()
        },
        transform: Transform::from_xyz(-half_w, half_h - side_wall_height / 2.0, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness / 2.0, side_wall_height / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // ⬅️ Pared izquierda abajo
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(wall_thickness, side_wall_height)),
            ..default()
        },
        transform: Transform::from_xyz(-half_w, -half_h + side_wall_height / 2.0, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness / 2.0, side_wall_height / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // ➡️ Pared derecha arriba
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(wall_thickness, side_wall_height)),
            ..default()
        },
        transform: Transform::from_xyz(half_w, half_h - side_wall_height / 2.0, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness / 2.0, side_wall_height / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // ➡️ Pared derecha abajo
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(wall_thickness, side_wall_height)),
            ..default()
        },
        transform: Transform::from_xyz(half_w, -half_h + side_wall_height / 2.0, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness / 2.0, side_wall_height / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // ⬆️ Pared superior
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(bounds.x, wall_thickness)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, half_h, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(bounds.x / 2.0, wall_thickness / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // ⬇️ Pared inferior
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(bounds.x, wall_thickness)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, -half_h - wall_thickness / 2.0, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(bounds.x / 2.0, wall_thickness / 2.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));

    // 🟣 Esquinas especiales (rotadas)
    let corner_size = 100.0;
    let half_corner = corner_size / 2.0;
    let corner_positions = [
        (550.0, 370.0),   // superior derecha
        (-555.0, 365.0),  // superior izquierda
        (550.0, -370.0),  // inferior derecha
        (-555.0, -375.0), // inferior izquierda
    ];

    for &(x, y) in &corner_positions {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::splat(corner_size)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, -10.0),
                rotation: Quat::from_rotation_z(45_f32.to_radians()),
                ..default()
            },
            ..default()
        })
            .insert(Collider::cuboid(half_corner, half_corner))
            .insert(RigidBody::Fixed)
            .insert(Restitution::coefficient(2.0));
    }
}
