use anyhow::Result;
use bevy::prelude::{
    default, App, Assets, Camera3dBundle, Color, Commands, DefaultPlugins, EventWriter, Input,
    KeyCode, Mesh, PbrBundle, PointLight, PointLightBundle, Quat, Query, Res, ResMut,
    StandardMaterial, Transform, Vec2, Vec3,
};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tweening::lens::{TransformRotationLens, TransformScaleLens};
use bevy_tweening::{
    Animator, AnimatorState, EaseFunction, RepeatCount, RepeatStrategy, Tracks, Tween,
    TweeningPlugin,
};
use rand::Rng;
use random_color::RandomColor;
use smooth_bevy_cameras::{
    controllers::fps::{ControlEvent, FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use std::time::Duration;

mod buildings;
mod mesh;

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::new(true))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .add_system(gui_controls)
        .add_system(camera_controls)
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Pass a path to a .geojson containing some polygons");
    }
    let path = args.pop().unwrap();

    let mut rng = rand::thread_rng();

    for polygon in mesh::load_polygons(&path).unwrap() {
        let mut builder = mesh::MeshBuilder::new();
        let height = rng.gen_range(200.0..500.0);
        buildings::extrude(polygon, height, &mut builder);

        let scale_height = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            TransformScaleLens {
                start: Vec3::new(0.01, 0.01, 0.01),
                end: Vec3::new(0.01, 0.04, 0.01),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        // TODO Need to express the spin around the polygon's center, oops
        let spin = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(5),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_y(180_f32.to_radians()),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(builder.build()).into(),
                material: materials.add(StandardMaterial {
                    base_color: bevy_color(RandomColor::new().hue(random_color::Color::Blue)),
                    cull_mode: None,
                    double_sided: true,
                    ..default()
                }),
                ..default()
            },
            Animator::new(Tracks::new([scale_height, spin])),
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                smoothing_weight: 0.0,
                ..default()
            },
            // eye
            Vec3::new(-20.0, 35.0, 5.0),
            // target
            Vec3::splat(0.0),
        ));
}

fn bevy_color(c: &mut RandomColor) -> Color {
    let [r, g, b] = c.to_rgb_array();
    Color::rgb_u8(r, g, b)
}

fn gui_controls(
    mut ctx: ResMut<EguiContext>,
    mut query: Query<&mut Animator<Transform>>,
    keyboard: Res<Input<KeyCode>>,
) {
    egui::Window::new("Controls").show(ctx.ctx_mut(), |ui| {
        // TODO This probably stops both
        if ui.button("Pause/resume height scaling").clicked()
            || keyboard.just_pressed(KeyCode::Space)
        {
            for mut x in &mut query {
                if x.state == AnimatorState::Playing {
                    x.stop();
                } else {
                    x.state = AnimatorState::Playing;
                }
            }
        }
    });
}

fn camera_controls(mut events: EventWriter<ControlEvent>, keyboard: Res<Input<KeyCode>>) {
    let translate_speed = 15.0;
    let rotate_speed = 2.0;

    if keyboard.pressed(KeyCode::Up) {
        events.send(ControlEvent::TranslateEye(translate_speed * Vec3::Z));
    }
    if keyboard.pressed(KeyCode::Down) {
        events.send(ControlEvent::TranslateEye(translate_speed * -Vec3::Z));
    }

    let translate = keyboard.pressed(KeyCode::LShift);
    for (key, translate_dir, rotate_dir) in [
        (KeyCode::Left, Vec3::X, -Vec2::X),
        (KeyCode::Right, -Vec3::X, Vec2::X),
        (KeyCode::A, -Vec3::Y, Vec2::Y),
        (KeyCode::Q, Vec3::Y, -Vec2::Y),
    ] {
        if keyboard.pressed(key) {
            if translate {
                events.send(ControlEvent::TranslateEye(translate_speed * translate_dir));
            } else {
                events.send(ControlEvent::Rotate(rotate_speed * rotate_dir));
            }
        }
    }
}
