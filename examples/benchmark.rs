use ashscript_solis_2d::prelude::*;
use bevy::{
    core_pipeline::tonemapping::Tonemapping, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, gizmos::light, input::mouse::MouseWheel, prelude::*
};
use bevy_egui::*;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "radiance cascade".into(),
                    ..default()
                }),
                ..default()
            }),
            bevy_egui::EguiPlugin,
            LightPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (setup, spawn_info_box))
        .add_systems(
            Update,
            (
                update,
                control_camera_zoom,
                config,
                diagnostics,
                monitor,
                control_camera_zoom,
                control_camera_movement,
            ),
        )
        .insert_resource(LightData::default())
        .run();
}

pub mod camera {
    pub const SPEED: f32 = 50.;
    pub const BOOST_SPEED: f32 = 100.;
    pub const Z_POS: f32 = 1000.;
    pub const MAX_SCALE: f32 = 10.;
    pub const MIN_SCALE: f32 = 1.;
}

pub mod map {
    pub const TILE_DIMENSIONS: f32 = 64.0;
    pub const MAP_DIMENSIONS: u32 = 2048; // 8192;
    pub const LIGHT_CHANCE: f32 = 0.1;
}

#[derive(Component)]
struct Spin(f32);

#[derive(Resource, Default)]
struct LightData {
    pub lights: u32,
    pub occluders: u32,
}

fn monitor(diagnostics: Res<DiagnosticsStore>) {
    let Some(_fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .map(|fps| fps.value())
        .flatten()
    else {
        return;
    };
}

fn setup(mut cmd: Commands, server: Res<AssetServer>, mut egui: EguiContexts, mut light_data: ResMut<LightData>) {
    cmd.spawn(RadianceCameraBundle {
        camera_bundle: Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))
                .looking_at(Vec3::default(), Vec3::Y),
            camera: Camera {
                clear_color: Color::BLACK.into(),
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        ..default()
    });

    for x in 0..=map::MAP_DIMENSIONS {
        for y in 0..=map::MAP_DIMENSIONS {
            match rand::random::<u32>() % (1. / map::LIGHT_CHANCE) as u32 {
                0 => {
                    cmd.spawn((
                        SpriteBundle {
                            texture: server.load("box.png"),
                            transform: Transform::from_xyz(
                                (x as f32) * map::TILE_DIMENSIONS,
                                (y as f32) * map::TILE_DIMENSIONS,
                                1.,
                            ),
                            ..default()
                        },
                        Emitter {
                            intensity: 1.,
                            color: Color::WHITE,
                            shape: SdfShape::Rect(Vec2::new(50., 25.)),
                        },
                        Spin(rand::random::<f32>()),
                    ));

                    light_data.lights += 1;
                }
                1 => {
                    cmd.spawn((
                        SpriteBundle {
                            texture: server.load("box.png"),
                            transform: Transform::from_xyz(
                                (x as f32) * map::TILE_DIMENSIONS,
                                (y as f32) * map::TILE_DIMENSIONS,
                                1.,
                            ),
                            ..default()
                        },
                        Emitter {
                            intensity: 1.,
                            color: Color::BLACK,
                            shape: SdfShape::Rect(Vec2::new(50., 25.)),
                        },
                        Spin(rand::random::<f32>()),
                    ));

                    light_data.occluders += 1;
                }
                _ => {
                    // do nothing
                }
            }
        }
    }
}

fn config(mut gi_config: Query<(&mut RadianceConfig, &mut RadianceDebug)>, mut egui: EguiContexts) {
    let Ok((mut cfg, mut debug)) = gi_config.get_single_mut() else {
        return;
    };

    egui::Window::new("Gi Config")
        .anchor(egui::Align2::RIGHT_TOP, [0., 0.])
        .show(egui.ctx_mut(), |ui| {
            ui.label("probe stride");
            ui.add(egui::Slider::new(&mut cfg.probe_base, (1)..=16));
            ui.label("cascade count");
            ui.add(egui::Slider::new(&mut cfg.cascade_count, (2)..=8));
            ui.label("interval");
            ui.add(egui::Slider::new(&mut cfg.interval, (0.1)..=10.));
            ui.label("scale");
            ui.add(egui::Slider::new(&mut cfg.scale_factor, (0.25)..=10.));
            ui.label("edge highlight");
            ui.add(egui::Slider::new(&mut cfg.edge_hightlight, (0.0)..=100.));

            flag_checkbox(GiFlags::DEBUG_SDF, ui, &mut debug, "SDF");
            flag_checkbox(GiFlags::DEBUG_VORONOI, ui, &mut debug, "VORONOI");
            flag_checkbox(GiFlags::DEBUG_MERGE1, ui, &mut debug, "MERGE0");
            flag_checkbox(GiFlags::DEBUG_MERGE0, ui, &mut debug, "MERGE1");
        });
}

#[derive(Default)]
struct Timings {
    count: u32,
    acc_fps: f64,
    acc_frame: f64,
    avg_fps: f64,
    avg_frame: f64,
}

fn diagnostics(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut timings: Local<Timings>,
    light_data: Res<LightData>,
) {
    let Some(fps_time) = diagnostics.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) else {
        return;
    };

    let Some(frame_time) = diagnostics.get_measurement(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
    else {
        return;
    };

    timings.acc_fps += fps_time.value;
    timings.acc_frame += frame_time.value;
    timings.count += 1;

    if timings.count == 100 {
        timings.avg_fps = timings.acc_fps * 0.01;
        timings.avg_frame = timings.acc_frame * 0.01;
        timings.count = 0;
        timings.acc_fps = 0.;
        timings.acc_frame = 0.;
    }

    egui::Window::new("Diagnostics")
        .anchor(egui::Align2::RIGHT_BOTTOM, [0., 0.])
        .show(egui.ctx_mut(), |ui| {
            ui.label(format!("FPS: {:.3}", timings.avg_fps));
            ui.label(format!("FRAME: {:.2}ms", timings.avg_frame));
            ui.label(format!("LIGHTS: {}", light_data.lights));
            ui.label(format!("OCCLUDERS: {}", light_data.occluders));
        });
}

fn flag_checkbox(bit: GiFlags, ui: &mut egui::Ui, flags: &mut GiFlags, label: &str) {
    let mut state = (*flags & bit) != GiFlags::DEFAULT;
    ui.checkbox(&mut state, label);
    if state {
        *flags |= bit;
    } else {
        *flags &= !bit;
    }
}

fn update(mut query: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut transform, spin)| {
        transform.rotation = Quat::from_rotation_z(time.elapsed_seconds() * spin.0);
    });
}

fn spawn_info_box(mut cmd: Commands) {
    let mut node = NodeBundle::default();
    node.style.width = Val::Percent(100.);
    node.style.height = Val::Percent(100.);
    node.style.align_items = AlignItems::End;
    node.style.justify_content = JustifyContent::Start;

    cmd.spawn(node).with_children(|cmd| {
        let mut node = NodeBundle::default();
        node.style.border = UiRect::all(Val::Px(4.));
        node.background_color = BackgroundColor(Color::BLACK);
        node.border_radius = BorderRadius::all(Val::Px(15.));
        node.style.padding = UiRect::all(Val::Px(10.));
        cmd.spawn(node).with_children(|cmd| {
            cmd.spawn(TextBundle::from_section(
                "[WASD]:Move [Scroll]:Zoom",
                TextStyle {
                    color: Color::WHITE,
                    font_size: 20.,
                    ..default()
                },
            ));
        });
    });
}

fn control_camera_zoom(
    mut cameras: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
    mut scroll_event_reader: EventReader<MouseWheel>,
) {
    let mut projection_delta = 0.;

    for event in scroll_event_reader.read() {
        projection_delta += event.y * 3.;
    }

    if projection_delta == 0. {
        return;
    }

    for mut camera in cameras.iter_mut() {
        camera.scale = (camera.scale - projection_delta * time.delta_seconds())
            .clamp(camera::MIN_SCALE, camera::MAX_SCALE);
    }
}

fn control_camera_movement(
    mut camera_current: Local<Vec2>,
    mut camera_target: Local<Vec2>,
    mut query_cameras: Query<&mut Transform, With<Camera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    _time: Res<Time>,
) {
    if keyboard.pressed(KeyCode::KeyW) {
        camera_target.y += camera::SPEED;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        camera_target.y -= camera::SPEED;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        camera_target.x -= camera::SPEED;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        camera_target.x += camera::SPEED;
    }

    // Smooth camera.
    let blend_ratio = 0.2;
    let movement = *camera_target - *camera_current;
    *camera_current += movement * blend_ratio;

    // Update all sprite cameras.
    for mut camera_transform in query_cameras.iter_mut() {
        camera_transform.translation.x = camera_current.x;
        camera_transform.translation.y = camera_current.y;
    }
}
