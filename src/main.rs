// maybe keep screenshot in memory
// add friction/velocity to zooming
extern crate raylib;
extern crate repng;
extern crate scrap;

use raylib::prelude::*;

use log::info;
use scrap::{Capturer, Display};
use std::env::temp_dir;
use std::fs;
use std::io::Cursor;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();
    use std::time::Instant;
    let now = Instant::now();

    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let (w, h) = (capturer.width(), capturer.height());
    let mut temp_screenshot_path = temp_dir();

    let file_name = format!("{}.png", "rustzoomerscreenshot");

    temp_screenshot_path.push(file_name);
    let mut png_buffer = Vec::new();

    loop {
        // Wait until there's a frame.

        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        //println!("Captured! Saving...");

        // Flip the ARGB image into a BGRA image.

        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let stride = buffer.len() / h;

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
            }
        }

        // Save the image.

        let cursor = Cursor::new(&mut png_buffer);
        repng::encode(cursor, w as u32, h as u32, &bitflipped).unwrap();
        break;
    }

    let (mut rl, thread) = raylib::init()
        .size(w as i32, h as i32)
        .title("Rust Screenshot")
        .build();

    let mut camera = Camera2D {
        target: Vector2 {
            x: w as f32 / 2.0,
            y: h as f32 / 2.0,
        },
        offset: Vector2 {
            x: w as f32 / 2.0,
            y: h as f32 / 2.0,
        },
        rotation: 0.0,
        zoom: 1.0,
    };

    rl.set_target_fps(144);

    rl.set_window_size(w as i32, h as i32);
    rl.toggle_fullscreen();
    let mut prev_mouse_pos = rl.get_mouse_position();
    let mut delta_scale = 0.0;
    let zoom_friction = 8.0;
    let zoom_speed = 2.5;

    let img = raylib::core::texture::Image::load_image_from_mem(
        ".png",
        &png_buffer,
        png_buffer.len() as i32,
    )
    .unwrap();

    let texture = rl.load_texture_from_image(&thread, &img).unwrap();
    let elapsed = now.elapsed();

    info!(
        "Time to take screenshot and initialize window was {:.2?}",
        elapsed
    );
    while !rl.window_should_close() {
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Q)
            || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        {
            let _ = fs::remove_file(temp_screenshot_path);
            break;
        }

        let mouse_pos = rl.get_mouse_position();
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 || delta_scale != 0.0 {
            // calculate the amount of zoom this frame
            delta_scale += wheel;

            let delta = rl.get_frame_time();
            // add zoom velocity to camera zoom
            camera.zoom = camera.zoom + delta_scale * delta * (zoom_speed * camera.zoom);
            if camera.zoom < 0.01 {
                camera.zoom = 0.01;
            }
            // apply friction on the zoom velocity
            delta_scale -= delta_scale * delta * zoom_friction;
            if delta_scale < 0.1 && delta_scale > -0.1 {
                delta_scale = 0.0;
            }
            camera.target = rl.get_screen_to_world2D(mouse_pos, camera);
            camera.offset = mouse_pos;
        }

        let delta = prev_mouse_pos - mouse_pos;
        prev_mouse_pos = mouse_pos;

        if rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_RIGHT_BUTTON) {
            camera.target = rl.get_screen_to_world2D(camera.offset + delta, camera);
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::DARKGRAY);

        {
            let mut mode_2d = d.begin_mode2D(camera);
            mode_2d.draw_texture(&texture, 0, 0, Color::WHITE);
        }

        d.draw_text("Press Q or ESCAPE to quit", 10, 10, 20, Color::GRAY);
        let zoom_amount = format!("zoom amount = {zoom:.2}", zoom = camera.zoom);
        d.draw_text(&zoom_amount, 10, 30, 20, Color::GRAY);
    }
}
