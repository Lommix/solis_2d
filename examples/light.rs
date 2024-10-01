use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::system::EntityCommands,
    input::mouse::MouseWheel,
    prelude::*,
    render::texture::ImageSamplerDescriptor,
};
use bevy_egui::*;
use lommix_light::prelude::*;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor::nearest(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "fragment radiant cascade".into(),
                        ..default()
                    }),
                    ..default()
                }),
            bevy_egui::EguiPlugin,
            LightPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (setup, debug_targets))
        .add_systems(
            Update,
            (
                update,
                scroll,
                move_camera,
                move_light,
                spawn_light,
                on_change,
                clear,
                config,
                monitor,
            ),
        )
        .run();
}

#[derive(Component)]
struct Spin(f32);

#[derive(Component)]
struct FollowMouse;

fn monitor(diagnostics: Res<DiagnosticsStore>) {
    let Some(_fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .map(|fps| fps.value())
        .flatten()
    else {
        return;
    };
}

fn setup(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default())
        .insert(Light2dCameraTag);

    for x in -4..=8 {
        for y in -4..=8 {
            cmd.spawn((
                SpriteBundle {
                    texture: server.load("box.png"),
                    transform: Transform::from_xyz((x as f32) * 400., (y as f32) * 400., 1.),
                    ..default()
                },
                Emitter {
                    intensity: 1.,
                    color: Color::BLACK,
                    shape: SdfShape::Rect(Vec2::new(50., 25.)),
                },
                Spin(rand::random::<f32>()),
            ));
        }
    }

    cmd.spawn((SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(2000.)),
            ..default()
        },
        texture: server.load("box.png"),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    },));

    cmd.spawn((
        Emitter {
            intensity: 1.,
            color: Color::WHITE,
            shape: SdfShape::Circle(200.),
        },
        FollowMouse,
        SpriteBundle {
            texture: server.load("lamp.png"),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
    ));
}

fn debug_targets(mut cmd: Commands, render_targets: Res<RenderTargets>) {
    cmd.spawn(NodeBundle {
        style: Style {
            border: UiRect::all(Val::Px(5.)),
            padding: UiRect::all(Val::Px(2.)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    })
    .with_children(|cmd| {
        let node = NodeBundle::default();
        cmd.spawn(node).with_children(|cmd| {
            cmd.preview(render_targets.sdf_target.clone(), 200., 200.);
            cmd.preview(render_targets.probe_target.clone(), 800., 200.);
        });

        let mut node = NodeBundle::default();
        node.style.flex_direction = FlexDirection::Column;
        cmd.spawn(node).with_children(|cmd| {
            for m in render_targets.merge_targets.iter() {
                cmd.preview(m.img.clone(), 300., 300.);
            }
        });
    });
}

fn config(mut gi_config: ResMut<GiConfig>, mut egui: EguiContexts) {
    egui::Window::new("Gi Config").show(egui.ctx_mut(), |ui| {
        ui.label("probe stride");
        ui.add(egui::Slider::new(&mut gi_config.probe_stride, (2)..=16));

        ui.label("cascade count");
        ui.add(egui::Slider::new(&mut gi_config.cascade_count, (1)..=8));

        ui.label("ray range");
        ui.add(egui::Slider::new(&mut gi_config.ray_range, (0.)..=1.));
        ui.label("scale");
        ui.add(egui::Slider::new(&mut gi_config.scale_factor, (1)..=10));

        flag_checkbox(GiFlags::DEBUG_SDF, ui, &mut gi_config, "SDF");
        flag_checkbox(GiFlags::DEBUG_VORONOI, ui, &mut gi_config, "VORONOI");
        flag_checkbox(GiFlags::DEBUG_LIGHT, ui, &mut gi_config, "LIGHT");
        flag_checkbox(GiFlags::DEBUG_BOUNCE, ui, &mut gi_config, "BOUNCE");
        flag_checkbox(GiFlags::DEBUG_PROBE, ui, &mut gi_config, "PROBE");
        flag_checkbox(GiFlags::DEBUG_MERGE, ui, &mut gi_config, "MERGE");
        // ui.image("file://assets/box.png");
        // ui.separator();
    });
}

fn on_change(mut cmd: Commands, config: Res<GiConfig>, mut local: Local<GiConfig>) {
    if *local != *config {
        local.clone_from(&config);
        cmd.trigger(ResizeEvent);
    }
}

fn flag_checkbox(bit: GiFlags, ui: &mut egui::Ui, cfg: &mut GiConfig, label: &str) {
    let mut state = (cfg.flags & bit) != GiFlags::DEFAULT;
    ui.checkbox(&mut state, label);
    if state {
        cfg.flags |= bit;
    } else {
        cfg.flags &= !bit;
    }
}

fn update(mut query: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut transform, spin)| {
        transform.rotation = Quat::from_rotation_z(time.elapsed_seconds() * spin.0);
    });
}

fn spawn_light(
    mut cmd: Commands,
    mut light: Query<(&mut Emitter, &Transform), With<FollowMouse>>,
    inputs: Res<ButtonInput<MouseButton>>,
    mut egui: EguiContexts,
    server: Res<AssetServer>,
) {
    let Ok((mut emitter, transform)) = light.get_single_mut() else {
        return;
    };

    if egui.ctx_mut().is_pointer_over_area() {
        return;
    }

    if inputs.just_pressed(MouseButton::Left) {
        cmd.spawn((
            emitter.clone(),
            SpriteBundle {
                texture: server.load("lamp.png"),
                transform: transform.clone(),
                ..default()
            },
        ));

        let color = Color::srgb(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );

        emitter.color = color;
    }

    if inputs.just_pressed(MouseButton::Right) {
        cmd.spawn((
            Emitter {
                shape: emitter.shape.clone(),
                color: Color::BLACK,
                intensity: 1.,
            },
            SpriteBundle {
                texture: server.load("lamp.png"),
                transform: transform.clone(),
                ..default()
            },
        ));
    }
}

fn clear(
    mut cmd: Commands,
    emitters: Query<Entity, (Without<FollowMouse>, Without<Spin>, With<Emitter>)>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::KeyR) {
        emitters
            .iter()
            .for_each(|entity| cmd.entity(entity).despawn_recursive());
    }
}

fn scroll(
    mut emitters: Query<&mut Emitter, With<FollowMouse>>,
    mut events: EventReader<MouseWheel>,
    inputs: Res<ButtonInput<KeyCode>>,
    mut multi: Local<f32>,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    let Ok(mut emitter) = emitters.get_single_mut() else {
        return;
    };

    let dir = event.y.signum();
    match inputs.pressed(KeyCode::ShiftLeft) {
        true => {
            emitter.intensity = (emitter.intensity + dir * 5.).max(0.);
        }
        false => {
            *multi = (*multi + dir).max(0.);
            emitter.shape = SdfShape::Circle(*multi);
            // let Ok(mut transform) = camera.get_single_mut() else {
            //     return;
            // };
            // transform.scale =
            //     (transform.scale + Vec3::splat(0.1) * event.y.signum()).max(Vec3::splat(0.1));
        }
    }
}

fn move_camera(mut camera: Query<&mut Transform, With<Camera>>, inputs: Res<ButtonInput<KeyCode>>) {
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };

    transform.translation += Vec3::new(
        (inputs.pressed(KeyCode::ArrowRight) as i32 - inputs.pressed(KeyCode::ArrowLeft) as i32)
            as f32,
        (inputs.pressed(KeyCode::ArrowUp) as i32 - inputs.pressed(KeyCode::ArrowDown) as i32)
            as f32,
        0.,
    ) * 2.;
}

fn move_light(
    window: Query<&Window>,
    camera: Query<Entity, With<Camera2d>>,
    light: Query<Entity, With<FollowMouse>>,
    mut transforms: Query<&mut Transform>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position().map(|pos| {
        let offset = transforms
            .get(camera.single())
            .unwrap()
            .translation
            .truncate();
        (pos - Vec2::new(window.width() / 2., window.height() / 2.)) * Vec2::new(1., -1.) + offset
    }) else {
        return;
    };

    let Some(mut light_transform) = light
        .get_single()
        .ok()
        .map(|entity| transforms.get_mut(entity).ok())
        .flatten()
    else {
        return;
    };

    light_transform.translation.x = cursor_pos.x;
    light_transform.translation.y = cursor_pos.y;
}

trait Preview {
    fn preview(&mut self, handle: Handle<Image>, width: f32, height: f32) -> EntityCommands;
}

impl Preview for ChildBuilder<'_> {
    fn preview(&mut self, handle: Handle<Image>, width: f32, height: f32) -> EntityCommands<'_> {
        self.spawn(ImageBundle {
            image: UiImage {
                texture: handle,
                ..default()
            },
            style: Style {
                width: Val::Px(width),
                height: Val::Px(height),
                ..default()
            },
            ..default()
        })
    }
}
