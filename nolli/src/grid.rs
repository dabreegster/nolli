use bevy::prelude::{Color, Component, Transform, Vec2, Vec3};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, GeometryBuilder};
use bevy_prototype_lyon::shapes;
use geo::{Contains, Point, Polygon, Rect};

#[derive(Clone, Component)]
pub struct Grid {
    // Keep in mind this is row-major, (y, x)
    inner: grid::Grid<Cell>,
    resolution_meters: f64,
    flood_frontier: Vec<(usize, usize)>,
}

#[derive(Clone, PartialEq)]
enum Cell {
    Empty,
    Building,
    Frontier,
    Flooded,
}

impl Grid {
    pub fn from_polygons(polygons: &[Polygon], bbox: Rect) -> Self {
        let resolution_meters = 10.0;
        let mut grid = Self {
            inner: grid::Grid::init(
                (bbox.height() / resolution_meters).ceil() as usize,
                (bbox.width() / resolution_meters).ceil() as usize,
                Cell::Empty,
            ),
            resolution_meters,
            flood_frontier: Vec::new(),
        };

        // TODO This is brute-force. Loop over each polygon, find the grid bbox, and fill out the
        // larger grid.
        for y in 0..grid.inner.rows() {
            for x in 0..grid.inner.cols() {
                let pt = grid.center_of_cell(x, y);
                if polygons.iter().any(|polygon| polygon.contains(&pt)) {
                    grid.inner[y][x] = Cell::Building;
                }
            }
        }

        grid
    }

    // TODO Switch to https://github.com/StarArawn/bevy_ecs_tilemap, or just render as one big
    // image/texture bitmap?
    pub fn render(&self) -> Vec<ShapeBundle> {
        // A bundle for each color is pretty awkward
        let mut frontier_builder = GeometryBuilder::new();
        let mut flooded_builder = GeometryBuilder::new();

        for y in 0..self.inner.rows() {
            for x in 0..self.inner.cols() {
                if !matches!(self.inner[y][x], Cell::Flooded | Cell::Frontier) {
                    continue;
                }
                let shape = shapes::Rectangle {
                    extents: Vec2::new(
                        self.resolution_meters as f32,
                        self.resolution_meters as f32,
                    ),
                    origin: shapes::RectangleOrigin::CustomCenter(pt_to_vec2(
                        self.center_of_cell(x, y),
                    )),
                };

                if self.inner[y][x] == Cell::Flooded {
                    flooded_builder = flooded_builder.add(&shape);
                } else {
                    frontier_builder = frontier_builder.add(&shape);
                }
            }
        }

        // Draw with a higher z-order than the buildings to prevent flicker
        vec![
            flooded_builder.build(
                DrawMode::Fill(FillMode::color(Color::hex("0F7BDB").unwrap())),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ),
            frontier_builder.build(
                DrawMode::Fill(FillMode::color(Color::hex("42FEFE").unwrap())),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ),
        ]
    }

    fn center_of_cell(&self, x: usize, y: usize) -> Point {
        Point::new(
            (0.5 + (x as f64)) * self.resolution_meters,
            (0.5 + (y as f64)) * self.resolution_meters,
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

    pub fn start_flood(&mut self, x: usize, y: usize) {
        self.flood_frontier.push((x, y));
    }

    pub fn flood(&mut self) {
        let mut next = Vec::new();
        for (x, y) in &self.flood_frontier {
            self.inner[*y][*x] = Cell::Flooded;
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
                    if self.inner[y][x] == Cell::Empty {
                        self.inner[y][x] = Cell::Frontier;
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
