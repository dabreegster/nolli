use anyhow::Result;
use bevy::prelude::{
    App, Camera2dBundle, Commands, Component, DefaultPlugins, Entity, Input, KeyCode, Query, Res,
    SystemSet, With,
};
use bevy::time::FixedTimestep;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;

use self::cursor_worldspace::CursorWorldspace;
use self::grid::Grid;

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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(do_flood),
        )
        .init_resource::<CursorWorldspace>()
        .add_system(cursor_worldspace::cursor_to_world)
        .run();

    Ok(())
}

// Just taggging the ShapeBundle to change it later
#[derive(Component)]
struct RenderGrid;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));

    let (buildings, bbox) =
        load_geo::load_buildings("/home/dabreegster/Downloads/export.geojson").unwrap();
    let grid = Grid::from_polygons(&buildings, bbox);

    for bundle in grid.render() {
        commands.spawn((bundle, RenderGrid));
    }
    commands.spawn(grid);
    commands.spawn(load_geo::render_polygons(buildings));
}

fn controls(keys: Res<Input<KeyCode>>, cursor: Res<CursorWorldspace>, mut query: Query<&mut Grid>) {
    if keys.just_pressed(KeyCode::Space) {
        let mut grid = query.single_mut();
        if let Some(pt) = cursor.0 {
            if let Some((x, y)) = grid.world_to_cell(pt) {
                println!("Starting flood from {x}, {y}");
                grid.start_flood(x, y);
            }
        }
    }
}

fn do_flood(
    mut query1: Query<&mut Grid>,
    query2: Query<Entity, With<RenderGrid>>,
    mut commands: Commands,
) {
    let mut grid = query1.single_mut();
    grid.flood();

    // TODO This is definitely not the way to re-render
    for entity in &query2 {
        commands.entity(entity).despawn();
    }
    for bundle in grid.render() {
        commands.spawn((bundle, RenderGrid));
    }
}
