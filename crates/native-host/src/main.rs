use std::{env, time::Duration};

use bevy::{
    input::{
        InputSystems,
        gamepad::{Gamepad, GamepadAxis, GamepadButton},
    },
    prelude::*,
    render::{
        RenderApp, RenderPlugin, RenderStartup,
        renderer::RenderAdapterInfo,
        settings::{Backends, WgpuSettings},
    },
};
use recovery_core::{ButtonEdges, Control, InputFrame};

const FIXED_STEP: Duration = Duration::from_nanos(16_666_667);
const DIAGNOSTIC_SPEED: f32 = 180.0;
const WINDOW_TITLE: &str = "ESP Ra.De. Psi native diagnostic — no recovered gameplay";

#[derive(Resource, Default)]
struct SmokeRun {
    remaining: Option<u64>,
    completed: u64,
}

impl SmokeRun {
    fn advance(&mut self) -> bool {
        if self.remaining == Some(0) {
            return false;
        }
        self.completed += 1;
        if let Some(remaining) = &mut self.remaining {
            *remaining -= 1;
            *remaining == 0
        } else {
            false
        }
    }
}

#[derive(Resource, Default)]
struct HostInput {
    current: InputFrame,
    pending_edges: ButtonEdges,
}

#[derive(Component)]
struct DiagnosticSquare;

fn main() {
    let smoke_frames = parse_smoke_frames();
    let mut wgpu_settings = WgpuSettings::default();
    #[cfg(target_os = "macos")]
    {
        wgpu_settings.backends = Some(Backends::METAL);
    }
    let render_plugin = RenderPlugin {
        render_creation: wgpu_settings.into(),
        ..default()
    };

    let mut app = App::new();
    app.insert_resource(SmokeRun {
        remaining: smoke_frames,
        completed: 0,
    })
    .init_resource::<HostInput>()
    .insert_resource(Time::<Fixed>::from_duration(FIXED_STEP))
    .add_plugins(DefaultPlugins.set(render_plugin).set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.into(),
            ..default()
        }),
        ..default()
    }))
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, capture_host_input.after(InputSystems))
    .add_systems(FixedUpdate, diagnostic_movement)
    .add_systems(FixedUpdate, count_smoke_ticks.after(diagnostic_movement));
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(RenderStartup, report_adapter);
    }
    app.run();
}

fn parse_smoke_frames() -> Option<u64> {
    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        return None;
    };
    if first == "--help" || first == "-h" {
        println!(
            "ESP Ra.De. Psi native diagnostic host\n\nUsage: native-host [--smoke-frames COUNT]"
        );
        std::process::exit(0);
    }
    if first != "--smoke-frames" {
        eprintln!("unknown argument: {first}");
        std::process::exit(2);
    }
    let value = args.next().unwrap_or_else(|| {
        eprintln!("--smoke-frames requires a positive frame count");
        std::process::exit(2);
    });
    if args.next().is_some() {
        eprintln!("unexpected trailing argument");
        std::process::exit(2);
    }
    let count = value.parse::<u64>().unwrap_or_else(|_| {
        eprintln!("invalid --smoke-frames value: {value}");
        std::process::exit(2);
    });
    if count == 0 {
        eprintln!("--smoke-frames must be greater than zero");
        std::process::exit(2);
    }
    Some(count)
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.95), Vec2::splat(32.0)),
        DiagnosticSquare,
    ));
    info!("Diagnostic host only: square motion is synthetic, not recovered game behavior.");
}

fn report_adapter(adapter: Res<RenderAdapterInfo>) {
    println!(
        "render adapter selected: name={} backend={:?} device_type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
}

fn capture_host_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut input: ResMut<HostInput>,
) {
    let mut frame = InputFrame::default();
    frame = frame.with_down(
        Control::Up,
        keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW),
    );
    frame = frame.with_down(
        Control::Down,
        keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS),
    );
    frame = frame.with_down(
        Control::Left,
        keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA),
    );
    frame = frame.with_down(
        Control::Right,
        keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD),
    );
    frame = frame.with_down(Control::Confirm, keys.pressed(KeyCode::Enter));
    frame = frame.with_down(Control::Cancel, keys.pressed(KeyCode::Escape));
    frame = frame.with_down(Control::Start, keys.pressed(KeyCode::Space));

    let mut axis_x = 0.0f32;
    let mut axis_y = 0.0f32;
    if let Some(gamepad) = gamepads.iter().next() {
        axis_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        axis_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        frame = frame.with_down(
            Control::Confirm,
            frame.is_down(Control::Confirm) || gamepad.pressed(GamepadButton::South),
        );
        frame = frame.with_down(
            Control::Cancel,
            frame.is_down(Control::Cancel) || gamepad.pressed(GamepadButton::East),
        );
        frame = frame.with_down(
            Control::Start,
            frame.is_down(Control::Start) || gamepad.pressed(GamepadButton::Start),
        );
    }
    frame.axis_x = (axis_x.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
    frame.axis_y = (axis_y.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;

    input.pending_edges = input
        .pending_edges
        .merge(ButtonEdges::between(input.current, frame));
    input.current = frame;
}

fn diagnostic_movement(
    mut host_input: ResMut<HostInput>,
    time: Res<Time<Fixed>>,
    mut square: Query<&mut Transform, With<DiagnosticSquare>>,
) {
    let frame = host_input.current;
    let mut x = i32::from(frame.is_down(Control::Right)) - i32::from(frame.is_down(Control::Left));
    let mut y = i32::from(frame.is_down(Control::Up)) - i32::from(frame.is_down(Control::Down));
    x += i32::from(frame.axis_x.unsigned_abs() > 6_553) * i32::from(frame.axis_x > 0)
        - i32::from(frame.axis_x.unsigned_abs() > 6_553) * i32::from(frame.axis_x < 0);
    y += i32::from(frame.axis_y.unsigned_abs() > 6_553) * i32::from(frame.axis_y > 0)
        - i32::from(frame.axis_y.unsigned_abs() > 6_553) * i32::from(frame.axis_y < 0);
    if host_input.pending_edges.pressed != 0 || host_input.pending_edges.released != 0 {
        debug!(
            "diagnostic host observed input edges: {:?}",
            host_input.pending_edges
        );
    }
    host_input.pending_edges = ButtonEdges::default();

    let delta =
        Vec2::new(x as f32, y as f32).normalize_or_zero() * DIAGNOSTIC_SPEED * time.delta_secs();
    for mut transform in &mut square {
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

fn count_smoke_ticks(mut smoke: ResMut<SmokeRun>, mut exit: MessageWriter<AppExit>) {
    if smoke.advance() {
        println!(
            "smoke run complete: {} fixed diagnostic frames",
            smoke.completed
        );
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::SmokeRun;

    #[test]
    fn bounded_run_exits_after_exactly_requested_ticks() {
        let mut smoke = SmokeRun {
            remaining: Some(2),
            completed: 0,
        };
        assert!(!smoke.advance());
        assert!(smoke.advance());
        assert!(!smoke.advance());
        assert_eq!(smoke.completed, 2);
    }
}
