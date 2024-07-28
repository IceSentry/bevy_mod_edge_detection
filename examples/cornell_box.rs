use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    color::palettes, core_pipeline::{
        fxaa::{Fxaa, Sensitivity},
        prepass::{DepthPrepass, NormalPrepass},
    }, diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin}, math::vec3, prelude::*, window::{PresentMode, WindowResolution}
};
use bevy_mod_edge_detection::{EdgeDetectionCamera, EdgeDetectionConfig, EdgeDetectionPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(720.0, 720.0),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
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
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.5, -8.75)
                .looking_at(vec3(0.0, 2.5, 0.0), Vec3::Y),
            ..default()
        },
        DepthPrepass,
        NormalPrepass,
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::Extreme,
            edge_threshold_min: Sensitivity::Extreme,
        },
        EdgeDetectionCamera,
    ));
}

fn setup_ui(mut commands: Commands) {
    let style = TextStyle {
        font_size: 16.0,
        color: Color::WHITE,
        ..default()
    };
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::from_style(style.clone()),
                TextSection::new(" fps\n", style.clone()),
                TextSection::from_style(style.clone()),
                TextSection::new(" ms", style),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            }),
        )
        .insert(BackgroundColor(Color::BLACK));
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
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: white.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    // top
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: white.clone(),
        transform: Transform::from_xyz(0.0, 5.0, 0.0).with_rotation(Quat::from_rotation_x(PI)),
        ..default()
    });
    // back
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: white,
        transform: Transform::from_xyz(0.0, 2.5, 2.5)
            .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        ..default()
    });
    // left
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: materials.add(StandardMaterial{base_color: Color::from(palettes::css::DIM_GREY), ..default()}),
        transform: Transform::from_xyz(2.5, 2.5, 0.0)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        ..default()
    });
    // right
    commands.spawn(PbrBundle {
        mesh: plane,
        material: materials.add(Color::from(palettes::css::GREEN)),
        transform: Transform::from_xyz(-2.5, 2.5, 0.0)
            .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
        ..default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0 - 0.005, 0.0)
            .with_rotation(Quat::from_rotation_x(PI)),
        ..default()
    });
}

fn spawn_boxes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_size = 1.25;
    let half_box_size = box_size / 2.0;

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(box_size, box_size * 2.0, box_size)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(half_box_size, half_box_size * 2.0, half_box_size)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_6)),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(box_size, box_size, box_size)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(-half_box_size, half_box_size, -half_box_size)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_6)),
        ..default()
    });
}

fn set_unlit(
    material_handles: Query<&Handle<StandardMaterial>>,
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
        if let Some(fps_smoothed) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(Diagnostic::smoothed)
        {
            text.sections[0].value = format!("{fps_smoothed:.1}");
        }

        if let Some(frame_time_smoothed) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(Diagnostic::smoothed)
        {
            text.sections[2].value = format!("{frame_time_smoothed:.3}");
        }
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
            t.translation.z -= speed * time.delta_seconds();
        }
        if key_input.pressed(KeyCode::KeyW) {
            t.translation.z += speed * time.delta_seconds();
        }
        if key_input.pressed(KeyCode::KeyD) {
            t.translation.x -= speed * time.delta_seconds();
        }
        if key_input.pressed(KeyCode::KeyA) {
            t.translation.x += speed * time.delta_seconds();
        }
        if key_input.pressed(KeyCode::KeyQ) {
            t.translation.y -= speed * time.delta_seconds();
        }
        if key_input.pressed(KeyCode::KeyE) {
            t.translation.y += speed * time.delta_seconds();
        }
    }
}
