use bevy::prelude::{Color, Component, Transform, Vec2};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, GeometryBuilder};
use bevy_prototype_lyon::shapes;
use geo::{Contains, Point, Polygon, Rect};

#[derive(Component)]
pub struct Grid {
    // Keep in mind this is row-major, (y, x)
    inner: grid::Grid<bool>,
    resolution_meters: f64,
    // TODO Switch to https://github.com/StarArawn/bevy_ecs_tilemap, or just render as one big
    // image/texture bitmap?
}

impl Grid {
    fn center_of_cell(&self, x: usize, y: usize) -> Point {
        Point::new(
            (0.5 + (x as f64)) * self.resolution_meters,
            (0.5 + (y as f64)) * self.resolution_meters,
        )
    }

    pub fn from_polygons(polygons: &[Polygon], bbox: Rect) -> Self {
        let resolution_meters = 10.0;
        let mut grid = Self {
            inner: grid::Grid::new(
                (bbox.height() / resolution_meters).ceil() as usize,
                (bbox.width() / resolution_meters).ceil() as usize,
            ),
            resolution_meters,
        };

        // TODO This is the brute-force way to do this. Fill out the grid for each polygon instead.
        for y in 0..grid.inner.rows() {
            for x in 0..grid.inner.cols() {
                let pt = grid.center_of_cell(x, y);
                if polygons.iter().any(|polygon| polygon.contains(&pt)) {
                    grid.inner[y][x] = true;
                }
            }
        }

        grid
    }

    pub fn render_unfilled(&self) -> ShapeBundle {
        let mut builder = GeometryBuilder::new();
        for y in 0..self.inner.rows() {
            for x in 0..self.inner.cols() {
                if !self.inner[y][x] {
                    builder = builder.add(&shapes::Rectangle {
                        extents: Vec2::new(
                            self.resolution_meters as f32,
                            self.resolution_meters as f32,
                        ),
                        origin: shapes::RectangleOrigin::CustomCenter(pt_to_vec2(
                            self.center_of_cell(x, y),
                        )),
                    });
                }
            }
        }

        builder.build(
            DrawMode::Fill(FillMode::color(Color::RED)),
            Transform::default(),
        )
    }

    pub fn world_to_cell(&self, world_pt: Vec2) -> Option<(usize, usize)> {
        if world_pt.x < 0.0 || world_pt.y < 0.0 {
            return None;
        }
        let x = (world_pt.x as f64 / self.resolution_meters).floor() as usize;
        let y = (world_pt.y as f64 / self.resolution_meters).floor() as usize;
        if x >= self.inner.cols() || y >= self.inner.rows() {
            return None;
        }
        Some((x, y))
    }

    // Caller should render_unfilled after this
    pub fn toggle(&mut self, x: usize, y: usize) {
        self.inner[y][x] = !self.inner[y][x];
    }
}

fn pt_to_vec2(pt: Point) -> Vec2 {
    Vec2::new(pt.x() as f32, pt.y() as f32)
}
