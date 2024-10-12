use bevy::{
    core_pipeline::tonemapping::Tonemapping, diagnostic::FrameTimeDiagnosticsPlugin,
    input::mouse::MouseWheel, prelude::*, render::primitives::Aabb,
};
use bevy_egui::*;
use ashscript_solis_2d::prelude::*;

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
            LightPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (setup, spawn_info_box))
        .add_systems(
            Update,
            (scroll, move_camera, move_light, spawn_light, clear, config),
        )
        .run();
}

#[derive(Component)]
struct FollowMouse;

fn setup(mut cmd: Commands) {
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

    cmd.spawn((
        SpatialBundle::default(),
        Emitter {
            intensity: 1.,
            color: Color::BLACK,
            shape: SdfShape::Rect(Vec2::new(50., 25.)),
        },
    ));
    cmd.spawn((
        Emitter {
            intensity: 1.,
            color: Color::WHITE,
            shape: SdfShape::Circle(200.),
        },
        FollowMouse,
        SpatialBundle::default(),
    ));
}

fn config(mut gi_config: Query<(&mut RadianceConfig, &mut RadianceDebug)>, mut egui: EguiContexts) {
    let Ok((mut gi_config, mut gi_debug)) = gi_config.get_single_mut() else {
        return;
    };

    egui::Window::new("Gi Config")
        .anchor(egui::Align2::RIGHT_TOP, [0., 0.])
        .show(egui.ctx_mut(), |ui| {
            ui.label("probe stride");
            ui.add(egui::Slider::new(&mut gi_config.probe_base, (1)..=16));
            ui.label("cascade count");
            ui.add(egui::Slider::new(&mut gi_config.cascade_count, (2)..=8));
            ui.label("interval");
            ui.add(egui::Slider::new(&mut gi_config.interval, (0.1)..=1000.));
            ui.label("scale");
            ui.add(egui::Slider::new(&mut gi_config.scale_factor, (0.25)..=10.));
            ui.label("edge highlight");
            ui.add(egui::Slider::new(&mut gi_config.edge_hightlight, (0.)..=10.));

            flag_checkbox(GiFlags::DEBUG_SDF, ui, &mut gi_debug, "SDF");
            flag_checkbox(GiFlags::DEBUG_VORONOI, ui, &mut gi_debug, "VORONOI");
            flag_checkbox(GiFlags::DEBUG_MERGE1, ui, &mut gi_debug, "MERGE0");
            flag_checkbox(GiFlags::DEBUG_MERGE0, ui, &mut gi_debug, "MERGE1");
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
            0.8, 0.,
            0.,
            // rand::random::<f32>(),
            // rand::random::<f32>(),
            // rand::random::<f32>(),
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
    emitters: Query<Entity, (Without<FollowMouse>, With<Emitter>)>,
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
            emitter.intensity = (emitter.intensity + dir * 0.1).max(0.);
        }
        false => {
            *multi = (*multi + dir).max(0.);
            emitter.shape = SdfShape::Circle(*multi);
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

    if inputs.just_pressed(KeyCode::KeyO) {
        transform.scale += Vec3::splat(0.05);
    }
    if inputs.just_pressed(KeyCode::KeyI) {
        transform.scale -= Vec3::splat(0.05);
    }
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

    let scale = transforms.get(camera.single()).unwrap().scale.x;
    let Some(mut light_transform) = light
        .get_single()
        .ok()
        .map(|entity| transforms.get_mut(entity).ok())
        .flatten()
    else {
        return;
    };

    light_transform.translation.x = cursor_pos.x / scale;
    light_transform.translation.y = cursor_pos.y / scale;
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
                "[Arrowkeys]:Move [I]:Zoom-in [O]:Zoom-out [Wheel]:Inc-size [Wheel+shift]:Inc-intensity [R]: clear screen",
                TextStyle {
                    color: Color::WHITE,
                    font_size: 20.,
                    ..default()
                },
            ));
        });
    });
}
