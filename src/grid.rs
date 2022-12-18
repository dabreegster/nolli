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
    flood_frontier: Vec<(usize, usize)>,
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
            flood_frontier: Vec::new(),
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
    pub fn start_flood(&mut self, x: usize, y: usize) {
        self.flood_frontier = vec![(x, y)];
    }

    pub fn flood(&mut self) {
        println!("Flooding {} values", self.flood_frontier.len());
        let mut next = Vec::new();
        for (x, y) in &self.flood_frontier {
            self.inner[*y][*x] = true;
        }
        for (x, y) in self.flood_frontier.drain(..) {
            let x = x as isize;
            let y = y as isize;

            for x in (x - 1)..=(x + 1) {
                for y in (y - 1)..=(y + 1) {
                    if x < 0 || y < 0 {
                        continue;
                    }
                    let x = x as usize;
                    let y = y as usize;
                    if x == self.inner.cols() || y == self.inner.rows() {
                        continue;
                    }
                    if !self.inner[y][x] {
                        // TODO Duplicates
                        next.push((x, y));
                    }
                }
            }
        }
        self.flood_frontier = next;
    }
}

fn pt_to_vec2(pt: Point) -> Vec2 {
    Vec2::new(pt.x() as f32, pt.y() as f32)
}
