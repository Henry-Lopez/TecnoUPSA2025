// Etiqueta para identificar la cámara del juego
#[derive(Component)]
pub struct GameCamera;

pub fn spawn_camera_and_background(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Cámara 2D que renderiza todo (UI incluida)
    commands.spawn((
        Camera2dBundle::default(),
        GameCamera, // 👈 etiqueta personalizada
    ));

    // Fondo de cancha (con z = -20 para no tapar texto)
    commands.spawn(SpriteBundle {
        texture: asset_server.load("cancha.png"),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -20.0),
            scale: Vec3::splat(1.0),
            ..default()
        },
        ..default()
    });
}
/// Elimina todas las cámaras activas para evitar duplicadas y warnings.
use bevy::prelude::*;

/// Elimina todas las cámaras activas para evitar duplicadas y warnings.
// 2. En setup/camera.rs - Mejora la función cleanup_cameras para ser más robusta
pub fn cleanup_cameras(commands: &mut Commands, query: Query<Entity, With<Camera>>) {
    let count = query.iter().count();
    if count > 0 {
        println!("🧹 Limpiando {} cámaras", count);
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    } else {
        println!("ℹ️ No se encontraron cámaras para limpiar");
    }
}