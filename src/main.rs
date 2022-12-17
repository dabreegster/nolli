use anyhow::Result;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;
use geo::{
    BoundingRect, Geometry, GeometryCollection, HaversineDistance, MapCoordsInPlace, Point, Polygon,
};
use geojson::GeoJson;

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .run();

    Ok(())
}

#[derive(Component)]
struct Buildings;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));

    let buildings = load_buildings("/home/dabreegster/Downloads/export.geojson").unwrap();

    let mut builder = GeometryBuilder::new();
    for geo_polygon in buildings {
        let bevy_polygon = bevy_prototype_lyon::shapes::Polygon {
            points: geo_polygon
                .exterior()
                .coords()
                .map(|pt| Vec2::new(pt.x as f32, pt.y as f32))
                .collect(),
            closed: true,
        };
        builder = builder.add(&bevy_polygon);
    }

    commands.spawn(builder.build(
        DrawMode::Fill(FillMode::color(Color::CYAN)),
        Transform::default(),
    ));
}

/// Load polygons from a GeoJSON file and transform to Mercator
fn load_buildings(path: &str) -> Result<Vec<Polygon>> {
    let geojson = std::fs::read_to_string(path)?.parse::<GeoJson>()?;
    let mut collection: GeometryCollection<f64> = geojson::quick_collection(&geojson)?;
    let top_left: Point = collection.bounding_rect().unwrap().min().into();

    collection.map_coords_in_place(|c| {
        let x = Point::new(c.x, top_left.y()).haversine_distance(&top_left);
        let y = Point::new(top_left.x(), c.y).haversine_distance(&top_left);
        (x, y).into()
    });

    let mut polygons = Vec::new();
    for geom in collection {
        if let Geometry::Polygon(polygon) = geom {
            polygons.push(polygon);
        }
    }
    Ok(polygons)
}
