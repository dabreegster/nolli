use bevy::prelude::Vec3;
use geo::Polygon;

use crate::mesh::MeshBuilder;

pub fn extrude(polygon: Polygon, y2: f32, builder: &mut MeshBuilder) {
    let down = Vec3::NEG_Y;
    let up = Vec3::new(0.0, 1.0, 0.0);

    let y1 = 0.0;

    // Floor
    builder.triangulate_polygon(&polygon, y1, down);

    // Ceiling
    builder.triangulate_polygon(&polygon, y2, up);

    // For every line along the polygon, add a rectangular wall
    for line in polygon.exterior().lines() {
        let corner1 = Vec3::new(line.start.x as f32, y1, line.start.y as f32);
        let corner2 = Vec3::new(line.end.x as f32, y1, line.end.y as f32);
        let corner3 = Vec3::new(line.end.x as f32, y2, line.end.y as f32);
        let corner4 = Vec3::new(line.start.x as f32, y2, line.start.y as f32);

        // Now let's go fetch our buddy Norm
        let bottom_line = corner2 - corner1;
        let up_line = corner3 - corner2;
        let normal = bottom_line.cross(up_line).normalize();

        builder.add_quad([corner1, corner2, corner3, corner4], normal);
    }
    // TODO Interiors. The normal is reversed
}
