use anyhow::Result;
use bevy::prelude::Mesh;
use bevy_earcutr::{EarcutrInput, PolygonMeshBuilder};
use geo::{
    BoundingRect, Geometry, GeometryCollection, HaversineDistance, MapCoordsInPlace, Point, Polygon,
};
use geojson::GeoJson;

/// Load polygons from a GeoJSON file and transform to Mercator
pub fn load_buildings(path: &str) -> Result<Vec<Polygon>> {
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

    let mut polygons = Vec::new();
    for geom in collection {
        if let Geometry::Polygon(polygon) = geom {
            polygons.push(polygon);
        }
    }
    Ok(polygons)
}

pub fn polygons_to_mesh(polygons: Vec<Polygon>) -> Mesh {
    let mut builder = PolygonMeshBuilder::new();
    for geo_polygon in polygons {
        builder.add_earcutr_input(EarcutrInput {
            vertices: geo_polygon
                .exterior()
                .coords()
                .flat_map(|c| vec![c.x, c.y])
                .collect(),
            interior_indices: vec![],
        });
    }
    let mut mesh = builder.build().unwrap();

    let normals = vec![[0.0, 0.0, 1.0]; mesh.count_vertices()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
