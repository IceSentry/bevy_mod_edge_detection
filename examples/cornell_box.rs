use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    core_pipeline::{
        fxaa::{Fxaa, Sensitivity},
        prepass::{DepthPrepass, NormalPrepass},
    },
    diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_mod_edge_detection::{EdgeDetectionConfig, EdgeDetectionPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(720.0, 720.0),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(EdgeDetectionPlugin)
        .init_resource::<EdgeDetectionConfig>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_ui)
        .add_startup_system(spawn_cornell_box)
        .add_startup_system(spawn_boxes)
        .add_startup_system(set_unlit.in_base_set(StartupSet::PostStartup))
        .add_system(update_diagnostic_display)
        .add_system(update_config)
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
    ));
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraMono-Medium.ttf");
    let style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::WHITE,
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
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },

                ..default()
            }),
        )
        .insert(BackgroundColor(Color::BLACK.with_a(0.75)));
}

fn spawn_cornell_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let white = materials.add(Color::WHITE.into());
    let plane_size = 5.0;
    let plane = meshes.add(shape::Plane::from_size(plane_size).into());

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
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(2.5, 2.5, 0.0)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        ..default()
    });
    // right
    commands.spawn(PbrBundle {
        mesh: plane,
        material: materials.add(Color::GREEN.into()),
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
        mesh: meshes.add(shape::Box::new(box_size, box_size * 2.0, box_size).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(half_box_size, half_box_size * 2.0, half_box_size)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_6)),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube { size: box_size }.into()),
        material: materials.add(Color::WHITE.into()),
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

fn update_diagnostic_display(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    for mut text in &mut query {
        if let Some(fps_smoothed) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(Diagnostic::smoothed)
        {
            text.sections[0].value = format!("{fps_smoothed:.1}");
        }

        if let Some(frame_time_smoothed) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(Diagnostic::smoothed)
        {
            text.sections[2].value = format!("{frame_time_smoothed:.3}");
        }
    }
}

fn update_config(mut config: ResMut<EdgeDetectionConfig>, key_input: Res<Input<KeyCode>>) {
    if key_input.just_pressed(KeyCode::X) {
        config.debug = (config.debug + 1) % 2;
        println!("debug: {:?}", config.debug != 0);
    }
    if key_input.just_pressed(KeyCode::C) {
        config.enabled = (config.enabled + 1) % 2;
        println!("enabled: {:?}", config.enabled != 0);
    }
}
