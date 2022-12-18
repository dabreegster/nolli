use bevy::prelude::{Camera, GlobalTransform, Query, Res, ResMut, Resource, Vec2};
use bevy::render::camera::RenderTarget;
use bevy::window::Windows;

#[derive(Resource, Default, Debug)]
pub struct CursorWorldspace(pub Option<Vec2>);

// From https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn cursor_to_world(
    windows: Res<Windows>,
    query: Query<(&Camera, &GlobalTransform)>,
    mut cursor: ResMut<CursorWorldspace>,
) {
    // Assume exactly one camera
    let (camera, camera_transform) = query.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = window.cursor_position() {
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos: Vec2 = world_pos.truncate();
        cursor.0 = Some(world_pos);
    } else {
        cursor.0 = None;
    }
}
