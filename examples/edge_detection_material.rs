use bevy::{
    core_pipeline::{
        fxaa::{Fxaa, Sensitivity},
        prepass::{DeferredPrepass, DepthPrepass, NormalPrepass},
    },
    prelude::*,
};
use bevy_mod_edge_detection::prelude::*;

fn main() {
    App::new()
        // MSAA currently doesn't work correctly with the plugin
        .insert_resource(Msaa::Off)
        // EdgeDetectionConfig is initialized in the plugin, or can be added before to override it, or modified during runtime
        .init_resource::<EdgeDetectionConfig>()
        .add_plugins((DefaultPlugins, EdgeDetectionPlugin))
        .add_plugins(MaterialPlugin::<StandardEdgeDetectionMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_things, keyboard_configuration_changing))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut extmaterials: ResMut<Assets<StandardEdgeDetectionMaterial>>,
) {
    // set up the camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // The edge detection effect requires the depth, normal as deferred prepass
        NormalPrepass,
        DepthPrepass,
        DeferredPrepass,
        // Add some anti-aliasing because the lines can be really harsh otherwise
        // This isn't required, but some form of AA is recommended
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::Extreme,
            edge_threshold_min: Sensitivity::Extreme,
        },
        bevy_mod_edge_detection::EdgeDetectionCamera,
        // could also use the bundle of things needed for this effect
        // bevy_mod_edge_detection::EdgeDetectionMarkerBundle,
    ));

    // set up basic scene

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    let stdmat: StandardMaterial = Color::rgb_u8(124, 144, 255).into();
    // cube with extmat
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: extmaterials.add(stdmat.clone().to_edge_material()),
            transform: Transform::from_xyz(0.0, 2., 0.0),
            ..default()
        },
        Rotate,
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(stdmat),
            transform: Transform::from_xyz(0.0, 0.7, 0.0),
            ..default()
        },
        Rotate,
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_sections([TextSection::new(
                    "Space: toggle fullscreen/entity edge detection",
                    style.clone(),
                )]),
                ..default()
            });
            parent.spawn(TextBundle {
                text: Text::from_sections([TextSection::new(
                    "A/S: increase/decrease thickness",
                    style.clone(),
                )]),
                ..default()
            });
        });
}

#[derive(Component)]
struct Rotate;

fn rotate_things(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut t in query.iter_mut() {
        t.rotate_x(0.01);
        t.rotate_y(0.005);
    }
}
fn keyboard_configuration_changing(
    button: Res<ButtonInput<KeyCode>>,
    mut conf: ResMut<EdgeDetectionConfig>,
) {
    if button.just_pressed(KeyCode::Space) {
        if conf.full_screen_enabled > 0 {
            conf.full_screen_enabled = 0;
        } else {
            conf.full_screen_enabled = 1;
        }
    }
    if button.just_pressed(KeyCode::KeyA) {
        conf.thickness += 1.;
    }

    if button.just_pressed(KeyCode::KeyS) {
        conf.thickness -= 1.;
    }
}
