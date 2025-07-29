use bevy::{
    anti_aliasing::fxaa::{Fxaa, Sensitivity},
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    prelude::*,
};
use bevy_mod_edge_detection::{EdgeDetectionCamera, EdgeDetectionConfig, EdgeDetectionPlugin};

fn main() {
    App::new()
        // MSAA currently doesn't work correctly with the plugin
        .add_plugins((DefaultPlugins, EdgeDetectionPlugin))
        .init_resource::<EdgeDetectionConfig>()
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // set up the camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        // The edge detection effect requires the depth and normal prepass
        DepthPrepass,
        NormalPrepass,
        // Add some anti-aliasing because the lines can be really harsh otherwise
        // This isn't required, but some form of AA is recommended
        Msaa::Off,
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::Extreme,
            edge_threshold_min: Sensitivity::Extreme,
        },
        EdgeDetectionCamera,
    ));

    // set up basic scene

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.487, 0.564, 1.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
