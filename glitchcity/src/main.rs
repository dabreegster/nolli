use anyhow::Result;
use bevy::prelude::{
    default, App, Assets, Camera3dBundle, Color, Commands, DefaultPlugins, Mesh, PbrBundle,
    PointLight, PointLightBundle, ResMut, StandardMaterial, Transform, Vec3,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

mod load_geo;

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Pass a path to a .geojson containing some polygons");
    }
    let path = args.pop().unwrap();

    let buildings = load_geo::load_buildings(&path).unwrap();

    commands.spawn(PbrBundle {
        mesh: meshes.add(load_geo::polygons_to_mesh(buildings)).into(),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("601865").unwrap(),
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        // They're way too huge otherwise
        transform: Transform::from_scale(Vec3::splat(0.01)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 15.0,
                ..default()
            },
            // eye
            Vec3::new(-2.0, 5.0, 5.0),
            // target
            Vec3::new(0., 0., 0.),
        ));
}
