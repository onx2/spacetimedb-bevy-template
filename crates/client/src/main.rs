mod module_bindings;
mod stdb;

use bevy::prelude::*;
use bevy_stdb::prelude::*;
use module_bindings::*;
use stdb::*;

#[derive(Component, Debug, Default)]
pub struct PlayerMarker;

#[derive(Component, Debug, Default)]
pub struct NetTransform {
    x: f32,
    y: f32,
}

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: String::from("SpacetimeDB + Bevy template"),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        );

        app.add_plugins(MyStdbPlugin);

        app.add_systems(Startup, (spawn_camera, helper_text.spawn()));

        app.add_systems(PreUpdate, subscribe_on_connect);
        app.add_systems(
            PreUpdate,
            (subscribe_on_connect, sync_position, spawn_player).run_if(resource_exists::<StdbConn>),
        );
        app.add_systems(
            Update,
            (interpolate, handle_move_request).run_if(resource_exists::<StdbConn>),
        );
    }
}

fn helper_text() -> impl Scene {
    bsn! {
        Text::new("Use WASD to move.")
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn subscribe_on_connect(mut connected_msgs: ReadStdbConnectedMessage, mut subs: ResMut<StdbSubs>) {
    for msg in connected_msgs.read() {
        subs.subscribe_query(SubKey::Player, |q| {
            q.from.player().r#where(|p| p.identity.eq(msg.identity))
        });
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut insert_player_msgs: ReadInsertMessage<Player>,
) {
    for msg in insert_player_msgs.read() {
        commands.spawn((
            PlayerMarker,
            Mesh2d(meshes.add(Circle::new(20.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.4, 1.0))),
            Transform::from_xyz(msg.row.x, msg.row.y, 0.0),
            NetTransform {
                x: msg.row.x,
                y: msg.row.y,
            },
        ));
    }
}

/// Interpolate the rendered position of the player toward the server authority's position
///
/// NOTE: Single will silently fail if there are more than one of the type found.
fn interpolate(
    time: Res<Time>,
    player: Single<(&mut Transform, &NetTransform), With<PlayerMarker>>,
    window: Single<&Window>,
) {
    let dt = time.delta_secs();
    let (mut transform, net_transform) = player.into_inner();
    let target = Vec3::new(net_transform.x, net_transform.y, transform.translation.z);

    let distance = transform.translation.distance(target);

    // If the distance is larger than half the screen width, assume screen edge wrapping.
    let wrap_threshold = window.width() / 2.0;
    if distance > wrap_threshold {
        transform.translation = target;
    } else {
        transform.translation.smooth_nudge(&target, 18.0, dt);
    }
}

/// Store the server authority position on the Player component for use in interpolate system
fn sync_position(
    mut player: Single<&mut NetTransform, With<PlayerMarker>>,
    mut msgs: ReadUpdateMessage<Player>,
) {
    for msg in msgs.read() {
        player.x = msg.new.x;
        player.y = msg.new.y;
    }
}

const MOVE_SPEED: f32 = 2_000.0;
fn handle_move_request(
    conn: Res<StdbConn>,
    player: Single<&Transform, With<PlayerMarker>>,
    window: Single<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    let step = direction.normalize() * MOVE_SPEED * time.delta_secs();
    let half_w = window.width() * 0.5;
    let half_h = window.height() * 0.5;

    let _ = conn.reducers().move_player(
        (player.translation.x + step.x + half_w).rem_euclid(window.width()) - half_w,
        (player.translation.y + step.y + half_h).rem_euclid(window.height()) - half_h,
    );
}
