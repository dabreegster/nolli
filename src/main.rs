use anyhow::Result;
use bevy::prelude::{App, Camera2dBundle, Color, Commands, DefaultPlugins, Transform, Vec2};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, GeometryBuilder, ShapePlugin};
use bevy_prototype_lyon::shapes;
use geo::{
    BoundingRect, Contains, Coord, Geometry, GeometryCollection, HaversineDistance,
    MapCoordsInPlace, Point, Polygon, Rect,
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

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));

    let (buildings, bbox) = load_buildings("/home/dabreegster/Downloads/export.geojson").unwrap();
    let grid = Grid::from_polygons(&buildings, bbox);

    commands.spawn(grid.render_unfilled());
    commands.spawn(render_polygons(buildings));
}

fn render_polygons(polygons: Vec<Polygon>) -> ShapeBundle {
    let mut builder = GeometryBuilder::new();
    for geo_polygon in polygons {
        let bevy_polygon = shapes::Polygon {
            points: geo_polygon.exterior().coords().map(coord_to_vec2).collect(),
            closed: true,
        };
        builder = builder.add(&bevy_polygon);
    }
    builder.build(
        DrawMode::Fill(FillMode::color(Color::CYAN)),
        Transform::default(),
    )
}

/// Load polygons from a GeoJSON file and transform to Mercator
fn load_buildings(path: &str) -> Result<(Vec<Polygon>, Rect)> {
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

struct Grid {
    // Keep in mind this is row-major, (y, x)
    inner: grid::Grid<bool>,
    resolution_meters: f64,
}

impl Grid {
    fn center_of_cell(&self, x: usize, y: usize) -> Point {
        Point::new(
            (0.5 + (x as f64)) * self.resolution_meters,
            (0.5 + (y as f64)) * self.resolution_meters,
        )
    }

    fn from_polygons(polygons: &[Polygon], bbox: Rect) -> Self {
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

    fn render_unfilled(&self) -> ShapeBundle {
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
}

fn pt_to_vec2(pt: Point) -> Vec2 {
    Vec2::new(pt.x() as f32, pt.y() as f32)
}
fn coord_to_vec2(pt: &Coord) -> Vec2 {
    Vec2::new(pt.x as f32, pt.y as f32)
}
