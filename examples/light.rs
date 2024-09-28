use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
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
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                debug_targets,
                update,
                scroll,
                move_camera,
                move_light,
                spawn_light,
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
    let Some(fps) = diagnostics
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

fn debug_targets(
    mut cmd: Commands,
    render_targets: Res<RenderTargets>,
    mut delay: Local<f32>,
    time: Res<Time>,
) {
    *delay += time.delta_seconds();

    if *delay < 5. || *delay > 100. {
        return;
    }

    cmd.spawn(NodeBundle {
        border_color: BorderColor(Color::BLACK),
        background_color: BackgroundColor(Color::WHITE),
        style: Style {
            border: UiRect::all(Val::Px(5.)),
            padding: UiRect::all(Val::Px(2.)),
            ..default()
        },
        ..default()
    })
    .with_children(|cmd| {
        cmd.spawn(ImageBundle {
            image: UiImage {
                texture: render_targets.sdf_target.clone(),
                ..default()
            },
            style: Style {
                width: Val::Px(200.),
                height: Val::Px(200.),
                ..default()
            },
            ..default()
        });

        for tar in render_targets.merge_targets.iter() {
            cmd.spawn(ImageBundle {
                image: UiImage {
                    texture: tar.img.clone(),
                    ..default()
                },
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(200.),
                    ..default()
                },
                ..default()
            });
        }

        cmd.spawn(ImageBundle {
            image: UiImage {
                texture: render_targets.probe_target.clone(),
                ..default()
            },
            style: Style {
                width: Val::Px(400.),
                height: Val::Px(100.),
                ..default()
            },
            ..default()
        });
    });

    *delay = 500.;
}

fn config(mut gi_config: ResMut<GiConfig>, mut egui: EguiContexts) {
    egui::Window::new("Gi Config").show(egui.ctx_mut(), |ui| {
        ui.label("Sample Count");
        ui.add(egui::Slider::new(&mut gi_config.sample_count, 0..=50));
        ui.label("Probe Size");
        ui.add(egui::Slider::new(&mut gi_config.probe_size, (0.)..=1.));
        ui.label("Scale");
        ui.add(egui::Slider::new(&mut gi_config.scale, (1.)..=10.));

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
