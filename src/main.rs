use anyhow::Result;
use geo::{
    BoundingRect, Geometry, GeometryCollection, HaversineDistance, MapCoordsInPlace, Point, Polygon,
};
use geojson::GeoJson;

fn main() -> Result<()> {
    let buildings = load_buildings("/home/dabreegster/Downloads/export.geojson")?;
    println!("{} buildings", buildings.len());
    Ok(())
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
