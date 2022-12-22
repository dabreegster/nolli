use bevy::prelude::Vec3;
use geo::{CoordsIter, LineString, Polygon};

use crate::mesh::{MeshBuilder, Vertex};

pub fn extrude(polygon: Polygon, y2: f32, builder: &mut MeshBuilder) {
    let y1 = 0.0;

    // Floor
    builder.triangulate_polygon(&polygon, y1, Vec3::new(0.0, -1.0, 0.0));

    // Ceiling
    builder.triangulate_polygon(&polygon, y2, Vec3::new(0.0, 1.0, 0.0));
}
