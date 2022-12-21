use anyhow::Result;
use bevy::prelude::{
    default, App, Assets, Camera2dBundle, Color, ColorMaterial, Commands,
    DefaultPlugins, Mesh, ResMut
};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};

mod load_geo;

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Pass a path to a .geojson containing some polygons");
    }
    let path = args.pop().unwrap();

    let (buildings, _) = load_geo::load_buildings(&path).unwrap();

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(load_geo::polygons_to_mesh(buildings)).into(),
        material: materials
            .add(ColorMaterial::from(Color::hex("601865").unwrap()))
            .into(),
        ..default()
    });
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
