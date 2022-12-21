use anyhow::Result;
use bevy::prelude::{
    default, shape, App, Assets, Camera3dBundle, Color, Commands, DefaultPlugins, Mesh, PbrBundle,
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

    let (buildings, _) = load_geo::load_buildings(&path).unwrap();

    /*commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(load_geo::polygons_to_mesh(buildings)).into(),
        material: materials
            .add(Color::hex("601865").unwrap())
            .into(),
        ..default()
    });*/

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
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
