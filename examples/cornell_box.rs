use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    anti_aliasing::fxaa::{Fxaa, Sensitivity},
    color::palettes::css::{GREEN, RED, WHITE},
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_mod_edge_detection::{EdgeDetectionCamera, EdgeDetectionConfig, EdgeDetectionPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(720.0, 720.0),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            EdgeDetectionPlugin,
        ))
        .insert_resource(EdgeDetectionConfig {
            depth_threshold: 0.0,
            normal_threshold: 1.0,
            color_threshold: 0.0,
            debug: 0,
            ..default()
        })
        .add_systems(
            Startup,
            (setup_camera, setup_ui, spawn_cornell_box, spawn_boxes),
        )
        .add_systems(PostStartup, set_unlit)
        .add_systems(
            Update,
            (update_diagnostic_display, update_config, update_camera),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, -8.75).looking_at(vec3(0.0, 2.5, 0.0), Vec3::Y),
        DepthPrepass,
        NormalPrepass,
        Msaa::Off,
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::Extreme,
            edge_threshold_min: Sensitivity::Extreme,
        },
        EdgeDetectionCamera,
    ));
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Text::new(" fps\n ms"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(WHITE.into()),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ))
        .insert(BackgroundColor(Color::BLACK.with_alpha(0.75)));
}

fn spawn_cornell_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let white = materials.add(Color::WHITE);
    let plane_size = 5.0;
    let plane = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    // bottom
    commands.spawn((
        Mesh3d(plane.clone()),
        MeshMaterial3d(white.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // top
    commands.spawn((
        Mesh3d(plane.clone()),
        MeshMaterial3d(white.clone()),
        Transform::from_xyz(0.0, 5.0, 0.0).with_rotation(Quat::from_rotation_x(PI)),
    ));
    // back
    commands.spawn((
        Mesh3d(plane.clone()),
        MeshMaterial3d(white),
        Transform::from_xyz(0.0, 2.5, 2.5).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    ));
    // left
    commands.spawn((
        Mesh3d(plane.clone()),
        MeshMaterial3d(materials.add(Color::Srgba(RED))),
        Transform::from_xyz(2.5, 2.5, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));
    // right
    commands.spawn((
        Mesh3d(plane),
        MeshMaterial3d(materials.add(Color::Srgba(GREEN))),
        Transform::from_xyz(-2.5, 2.5, 0.0).with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0 - 0.005, 0.0).with_rotation(Quat::from_rotation_x(PI)),
    ));
}

fn spawn_boxes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_size = 1.25;
    let half_box_size = box_size / 2.0;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_size * 2.0, box_size))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(half_box_size, half_box_size * 2.0, half_box_size)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_6)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_size, box_size))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(-half_box_size, half_box_size, -half_box_size)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_6)),
    ));
}

fn set_unlit(
    material_handles: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for id in &material_handles {
        if let Some(material) = materials.get_mut(id) {
            material.unlit = true;
        }
    }
}

fn update_diagnostic_display(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        let fps_smoothed = if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(Diagnostic::smoothed)
        {
            fps
        } else {
            0.0
        };

        let frame_time_smoothed = if let Some(frame) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(Diagnostic::smoothed)
        {
            frame
        } else {
            0.0
        };

        text.0 = format!("{fps_smoothed:.0} fps\n{frame_time_smoothed:.1} ms");
    }
}

fn update_config(mut config: ResMut<EdgeDetectionConfig>, key_input: Res<ButtonInput<KeyCode>>) {
    if key_input.just_pressed(KeyCode::KeyX) {
        config.debug = (config.debug + 1) % 2;
        println!("debug: {:?}", config.debug != 0);
    }
    if key_input.just_pressed(KeyCode::KeyC) {
        config.enabled = (config.enabled + 1) % 2;
        println!("enabled: {:?}", config.enabled != 0);
    }
}

fn update_camera(
    key_input: Res<ButtonInput<KeyCode>>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let speed = 10.0;
    for mut t in &mut cam {
        if key_input.pressed(KeyCode::KeyS) {
            t.translation.z -= speed * time.delta_secs();
        }
        if key_input.pressed(KeyCode::KeyW) {
            t.translation.z += speed * time.delta_secs();
        }
        if key_input.pressed(KeyCode::KeyD) {
            t.translation.x -= speed * time.delta_secs();
        }
        if key_input.pressed(KeyCode::KeyA) {
            t.translation.x += speed * time.delta_secs();
        }
        if key_input.pressed(KeyCode::KeyQ) {
            t.translation.y -= speed * time.delta_secs();
        }
        if key_input.pressed(KeyCode::KeyE) {
            t.translation.y += speed * time.delta_secs();
        }
    }
}
