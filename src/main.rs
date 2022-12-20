use anyhow::Result;
use bevy::prelude::{
    default, App, Assets, Camera2dBundle, Color, ColorMaterial, Commands, Component,
    DefaultPlugins, Entity, Input, KeyCode, Mesh, ParamSet, Query, Res, ResMut, Resource,
    SystemSet, With,
};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::FixedTimestep;
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::WorldInspectorPlugin;
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
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(key_controls)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(do_flood),
        )
        .init_resource::<CursorWorldspace>()
        .add_system(cursor_worldspace::cursor_to_world)
        .insert_resource(FloodState { paused: false })
        .add_system(flood_controls)
        .run();

    Ok(())
}

// TODO These are all singletons. Should they just be a Resource?

// Just taggging the ShapeBundles to change them later
#[derive(Component)]
struct RenderGrid;

#[derive(Component)]
struct ActiveGrid;

#[derive(Component)]
struct OriginalGrid;

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

    let (buildings, bbox) = load_geo::load_buildings(&path).unwrap();
    let grid = Grid::from_polygons(&buildings, bbox);

    for bundle in grid.render() {
        commands.spawn((bundle, RenderGrid));
    }
    commands.spawn((grid.clone(), ActiveGrid));
    commands.spawn((grid, OriginalGrid));
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(load_geo::polygons_to_mesh(buildings)).into(),
        material: materials
            .add(ColorMaterial::from(Color::hex("601865").unwrap()))
            .into(),
        ..default()
    });
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

fn key_controls(
    keys: Res<Input<KeyCode>>,
    cursor: Res<CursorWorldspace>,
    mut query: Query<&mut Grid, With<ActiveGrid>>,
) {
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
    mut query1: Query<&mut Grid, With<ActiveGrid>>,
    query2: Query<Entity, With<RenderGrid>>,
    state: Res<FloodState>,
    mut commands: Commands,
) {
    if state.paused {
        return;
    }

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

#[derive(Resource)]
struct FloodState {
    paused: bool,
}

fn flood_controls(
    mut ctx: ResMut<EguiContext>,
    mut state: ResMut<FloodState>,
    mut set: ParamSet<(
        Query<&mut Grid, With<ActiveGrid>>,
        Query<&Grid, With<OriginalGrid>>,
    )>,
) {
    egui::Window::new("Controls").show(ctx.ctx_mut(), |ui| {
        if ui.button("Pause/resume").clicked() {
            state.paused = !state.paused;
        }
        if ui.button("Reset").clicked() {
            *set.p0().single_mut() = set.p1().single().clone();
            // TODO Re-render immediately, in case we're paused?
        }
    });
}
