mod astro;
mod catalog;

use std::f32::consts::{FRAC_PI_2, TAU};
use std::time::{SystemTime, UNIX_EPOCH};

use astro::{HorizontalCoordinates, Observer, equatorial_to_horizontal};
use catalog::BRIGHT_STARS;
use macroquad::prelude::*;

const BACKGROUND: Color = Color::new(0.012, 0.018, 0.035, 1.0);
const HORIZON_COLOR: Color = Color::new(0.24, 0.56, 0.58, 0.72);
const TEXT_PRIMARY: Color = Color::new(0.86, 0.91, 0.98, 1.0);
const TEXT_MUTED: Color = Color::new(0.48, 0.58, 0.72, 1.0);
const ACCENT: Color = Color::new(0.35, 0.88, 0.92, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Pi Constellation Mapper".to_owned(),
        window_width: 1_200,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(Clone, Copy)]
struct SkyCamera {
    yaw: f32,
    pitch: f32,
    field_of_view: f32,
}

impl SkyCamera {
    fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 30.0_f32.to_radians(),
            field_of_view: 60.0_f32.to_radians(),
        }
    }

    fn rotate(&mut self, delta: Vec2) {
        let radians_per_pixel = self.field_of_view / screen_height().max(320.0);
        self.yaw = (self.yaw - delta.x * radians_per_pixel).rem_euclid(TAU);
        self.pitch =
            (self.pitch + delta.y * radians_per_pixel).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }

    fn zoom_by(&mut self, factor: f32) {
        self.field_of_view =
            (self.field_of_view * factor).clamp(12.0_f32.to_radians(), 120.0_f32.to_radians());
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[derive(Default)]
struct PointerTracker {
    mouse_last: Option<Vec2>,
    mouse_start: Option<Vec2>,
    touch_last: Option<Vec2>,
    touch_start: Option<Vec2>,
    pinch_last_distance: Option<f32>,
}

impl PointerTracker {
    fn update(&mut self, camera: &mut SkyCamera) -> Option<Vec2> {
        let touch_positions: Vec<Vec2> = touches().iter().map(|touch| touch.position).collect();

        if !touch_positions.is_empty() {
            self.mouse_last = None;
            self.mouse_start = None;

            if touch_positions.len() >= 2 {
                let distance = touch_positions[0].distance(touch_positions[1]).max(1.0);
                if let Some(previous_distance) = self.pinch_last_distance {
                    camera.zoom_by((previous_distance / distance).clamp(0.82, 1.22));
                }
                self.pinch_last_distance = Some(distance);
                self.touch_last = None;
                return None;
            }

            self.pinch_last_distance = None;
            let position = touch_positions[0];
            if self.touch_start.is_none() {
                self.touch_start = Some(position);
            }
            if let Some(previous) = self.touch_last {
                camera.rotate(position - previous);
            }
            self.touch_last = Some(position);
            return None;
        }

        self.pinch_last_distance = None;
        if let (Some(start), Some(last)) = (self.touch_start.take(), self.touch_last.take())
            && start.distance(last) < 12.0
        {
            return Some(last);
        }

        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_pressed(MouseButton::Left) {
            self.mouse_start = Some(mouse);
            self.mouse_last = Some(mouse);
        }
        if is_mouse_button_down(MouseButton::Left) {
            if let Some(previous) = self.mouse_last {
                camera.rotate(mouse - previous);
            }
            self.mouse_last = Some(mouse);
        }
        if is_mouse_button_released(MouseButton::Left) {
            let start = self.mouse_start.take();
            self.mouse_last = None;
            if start.is_some_and(|position| position.distance(mouse) < 8.0) {
                return Some(mouse);
            }
        }

        let wheel = mouse_wheel().1;
        if wheel.abs() > f32::EPSILON {
            camera.zoom_by(0.88_f32.powf(wheel));
        }

        None
    }
}

#[derive(Clone, Copy)]
struct ProjectedStar {
    index: usize,
    position: Vec2,
    horizontal: HorizontalCoordinates,
}

#[macroquad::main(window_conf)]
async fn main() {
    simulate_mouse_with_touch(false);

    // Temporary development fallback. GPS coordinates will replace this observer.
    let observer = Observer {
        latitude_deg: 34.0522,
        longitude_deg: -118.2437,
    };
    let mut camera = SkyCamera::new();
    let mut pointer = PointerTracker::default();
    let mut selected_star: Option<usize> = None;
    let mut show_horizon = true;
    let mut fullscreen = false;

    loop {
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::H) {
            show_horizon = !show_horizon;
        }
        if is_key_pressed(KeyCode::R) {
            camera.reset();
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            camera.zoom_by(0.82);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            camera.zoom_by(1.22);
        }

        let tap_position = pointer.update(&mut camera);
        let unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |duration| duration.as_secs_f64());

        clear_background(BACKGROUND);

        let center = vec2(screen_width() * 0.5, screen_height() * 0.5);

        if show_horizon {
            draw_horizon(&camera);
        }

        let projected = draw_stars(&camera, observer, unix_seconds, selected_star);

        if let Some(tap) = tap_position {
            selected_star = projected
                .iter()
                .filter_map(|star| {
                    let distance = star.position.distance(tap);
                    (distance <= 24.0).then_some((star.index, distance))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(index, _)| index);
        }

        draw_crosshair(center);
        draw_hud(&camera, observer, selected_star, &projected);

        next_frame().await;
    }
}

fn draw_stars(
    camera: &SkyCamera,
    observer: Observer,
    unix_seconds: f64,
    selected_star: Option<usize>,
) -> Vec<ProjectedStar> {
    let mut projected = Vec::with_capacity(BRIGHT_STARS.len());

    for (index, star) in BRIGHT_STARS.iter().enumerate() {
        let horizontal = equatorial_to_horizontal(star, observer, unix_seconds);
        let direction = horizontal_direction(horizontal);
        let Some(position) = project_direction(direction, camera) else {
            continue;
        };

        let visibility = ((6.2 - star.magnitude) / 7.7).clamp(0.16, 1.0);
        let radius = 1.1 + visibility.powf(1.45) * 4.2;
        let (red, green, blue) = star.color_rgb;
        let color = Color::from_rgba(red, green, blue, (105.0 + visibility * 150.0) as u8);

        draw_circle(
            position.x,
            position.y,
            radius + 2.2,
            Color::new(color.r, color.g, color.b, 0.10),
        );
        draw_circle(position.x, position.y, radius, color);

        if selected_star == Some(index) {
            draw_circle_lines(position.x, position.y, radius + 7.0, 1.5, ACCENT);
        } else if star.magnitude <= 0.5 && camera.field_of_view.to_degrees() <= 90.0 {
            draw_text(
                star.name,
                position.x + 8.0,
                position.y - 6.0,
                15.0,
                TEXT_MUTED,
            );
        }

        projected.push(ProjectedStar {
            index,
            position,
            horizontal,
        });
    }

    projected
}

fn draw_horizon(camera: &SkyCamera) {
    let mut previous: Option<Vec2> = None;
    for azimuth in 0..=360 {
        let horizontal = HorizontalCoordinates {
            azimuth_deg: azimuth as f64,
            altitude_deg: 0.0,
        };
        let current = project_direction(horizontal_direction(horizontal), camera);

        if let (Some(from), Some(to)) = (previous, current)
            && from.distance(to) < screen_width().max(screen_height()) * 0.12
        {
            draw_line(from.x, from.y, to.x, to.y, 1.35, HORIZON_COLOR);
        }
        previous = current;
    }

    for (label, azimuth) in [("N", 0.0), ("E", 90.0), ("S", 180.0), ("W", 270.0)] {
        let horizontal = HorizontalCoordinates {
            azimuth_deg: azimuth,
            altitude_deg: 0.0,
        };
        if let Some(position) = project_direction(horizontal_direction(horizontal), camera) {
            draw_circle(
                position.x,
                position.y,
                10.0,
                Color::new(0.03, 0.08, 0.10, 0.82),
            );
            let width = measure_text(label, None, 14, 1.0).width;
            draw_text(
                label,
                position.x - width * 0.5,
                position.y + 5.0,
                14.0,
                HORIZON_COLOR,
            );
        }
    }
}

fn draw_crosshair(center: Vec2) {
    draw_line(
        center.x - 6.0,
        center.y,
        center.x + 6.0,
        center.y,
        1.0,
        Color::new(0.40, 0.52, 0.68, 0.6),
    );
    draw_line(
        center.x,
        center.y - 6.0,
        center.x,
        center.y + 6.0,
        1.0,
        Color::new(0.40, 0.52, 0.68, 0.6),
    );
}

fn draw_hud(
    camera: &SkyCamera,
    observer: Observer,
    selected_star: Option<usize>,
    projected: &[ProjectedStar],
) {
    let horizontal_fov =
        2.0 * ((camera.field_of_view * 0.5).tan() * (screen_width() / screen_height())).atan();
    draw_text("PI CONSTELLATION MAPPER", 20.0, 31.0, 22.0, TEXT_PRIMARY);
    draw_text(
        format!(
            "FOV V {:>3.0} deg / H {:>3.0} deg  |  {:.2}, {:.2}  |  {} FPS",
            camera.field_of_view.to_degrees(),
            horizontal_fov.to_degrees(),
            observer.latitude_deg,
            observer.longitude_deg,
            get_fps()
        ),
        20.0,
        53.0,
        15.0,
        TEXT_MUTED,
    );

    let help =
        "drag: explore   wheel/pinch: zoom   tap: inspect   H: horizon   R: reset   F: fullscreen";
    draw_text(help, 20.0, screen_height() - 18.0, 15.0, TEXT_MUTED);

    let Some(index) = selected_star else {
        return;
    };
    let Some(projected_star) = projected.iter().find(|star| star.index == index) else {
        return;
    };
    let star = &BRIGHT_STARS[index];
    let panel_width = 240.0;
    let panel_x = screen_width() - panel_width - 18.0;
    let panel_y = 18.0;
    draw_rectangle(
        panel_x,
        panel_y,
        panel_width,
        102.0,
        Color::new(0.025, 0.047, 0.085, 0.94),
    );
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_width,
        102.0,
        1.0,
        Color::new(0.20, 0.38, 0.54, 0.9),
    );
    draw_text(
        star.name,
        panel_x + 14.0,
        panel_y + 27.0,
        21.0,
        TEXT_PRIMARY,
    );
    draw_text(
        format!("apparent magnitude  {:.2}", star.magnitude),
        panel_x + 14.0,
        panel_y + 52.0,
        15.0,
        TEXT_MUTED,
    );
    draw_text(
        format!(
            "azimuth  {:>6.1} deg",
            projected_star.horizontal.azimuth_deg
        ),
        panel_x + 14.0,
        panel_y + 73.0,
        15.0,
        TEXT_MUTED,
    );
    draw_text(
        format!(
            "altitude {:>6.1} deg",
            projected_star.horizontal.altitude_deg
        ),
        panel_x + 14.0,
        panel_y + 92.0,
        15.0,
        if projected_star.horizontal.altitude_deg >= 0.0 {
            ACCENT
        } else {
            Color::new(0.88, 0.52, 0.48, 1.0)
        },
    );
}

fn horizontal_direction(horizontal: HorizontalCoordinates) -> Vec3 {
    let azimuth = horizontal.azimuth_deg.to_radians() as f32;
    let altitude = horizontal.altitude_deg.to_radians() as f32;
    vec3(
        altitude.cos() * azimuth.sin(),
        altitude.sin(),
        altitude.cos() * azimuth.cos(),
    )
}

fn project_direction(direction: Vec3, camera: &SkyCamera) -> Option<Vec2> {
    let forward = vec3(
        camera.pitch.cos() * camera.yaw.sin(),
        camera.pitch.sin(),
        camera.pitch.cos() * camera.yaw.cos(),
    );
    let right = vec3(camera.yaw.cos(), 0.0, -camera.yaw.sin());
    let up = forward.cross(right);

    let depth = direction.dot(forward);
    if depth <= 0.001 {
        return None;
    }

    // A rectilinear pinhole projection makes the display behave like a physical
    // window. When the vertical FOV matches the angle subtended by the screen at
    // the viewer's eye, star spacing on the display matches the sky behind it.
    let focal_length = screen_height() * 0.5 / (camera.field_of_view * 0.5).tan();
    let center = vec2(screen_width() * 0.5, screen_height() * 0.5);
    let position = vec2(
        center.x + direction.dot(right) / depth * focal_length,
        center.y - direction.dot(up) / depth * focal_length,
    );
    let margin = 32.0;

    (position.x >= -margin
        && position.x <= screen_width() + margin
        && position.y >= -margin
        && position.y <= screen_height() + margin)
        .then_some(position)
}
