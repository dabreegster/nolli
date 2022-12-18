use anyhow::Result;
use bevy::prelude::{Color, Transform, Vec2};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, GeometryBuilder};
use bevy_prototype_lyon::shapes;
use geo::{
    BoundingRect, Coord, Geometry, GeometryCollection, HaversineDistance, MapCoordsInPlace, Point,
    Polygon, Rect,
};
use geojson::GeoJson;

/// Load polygons from a GeoJSON file and transform to Mercator
pub fn load_buildings(path: &str) -> Result<(Vec<Polygon>, Rect)> {
    let geojson = std::fs::read_to_string(path)?.parse::<GeoJson>()?;
    let mut collection: GeometryCollection<f64> = geojson::quick_collection(&geojson)?;

    // Filter out non-polygons
    collection
        .0
        .retain(|geom| matches!(geom, Geometry::Polygon(_)));

    let top_left: Point = collection.bounding_rect().unwrap().min().into();

    collection.map_coords_in_place(|c| {
        let x = Point::new(c.x, top_left.y()).haversine_distance(&top_left);
        let y = Point::new(top_left.x(), c.y).haversine_distance(&top_left);
        (x, y).into()
    });
    let bbox = collection.bounding_rect().unwrap();

    let mut polygons = Vec::new();
    for geom in collection {
        if let Geometry::Polygon(polygon) = geom {
            polygons.push(polygon);
        }
    }
    Ok((polygons, bbox))
}

pub fn render_polygons(polygons: Vec<Polygon>) -> ShapeBundle {
    let mut builder = GeometryBuilder::new();
    for geo_polygon in polygons {
        let bevy_polygon = shapes::Polygon {
            points: geo_polygon.exterior().coords().map(coord_to_vec2).collect(),
            closed: true,
        };
        builder = builder.add(&bevy_polygon);
    }
    builder.build(
        DrawMode::Fill(FillMode::color(Color::hex("601865").unwrap())),
        Transform::default(),
    )
}

fn coord_to_vec2(pt: &Coord) -> Vec2 {
    Vec2::new(pt.x as f32, pt.y as f32)
}
