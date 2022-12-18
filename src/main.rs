use anyhow::Result;
use bevy::prelude::{App, Camera2dBundle, Commands, DefaultPlugins, Input, KeyCode, Res};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;

use self::cursor_worldspace::CursorWorldspace;

mod cursor_worldspace;
mod grid;
mod load_geo;

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .add_system(controls)
        .init_resource::<CursorWorldspace>()
        .add_system(cursor_worldspace::cursor_to_world)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));

    let (buildings, bbox) =
        load_geo::load_buildings("/home/dabreegster/Downloads/export.geojson").unwrap();
    let grid = grid::Grid::from_polygons(&buildings, bbox);

    commands.spawn(grid.render_unfilled());
    commands.spawn(load_geo::render_polygons(buildings));
}

fn controls(keys: Res<Input<KeyCode>>, cursor: Res<CursorWorldspace>) {
    if keys.just_pressed(KeyCode::Space) {
        println!("Space! At {:?}", cursor.0);
    }
}
