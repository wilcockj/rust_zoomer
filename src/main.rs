// maybe keep screenshot in memory
// add friction/velocity to zooming
extern crate raylib;
extern crate repng;
extern crate scrap;

use raylib::prelude::*;

use scrap::{Capturer, Display};
use std::cmp;
use std::fs;
use std::fs::File;
use std::io::ErrorKind::WouldBlock;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn main() {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());
    let screenshot_path = Path::new("./screenshot.png");

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

        println!("Captured! Saving...");

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

        repng::encode(
            File::create(screenshot_path).unwrap(),
            w as u32,
            h as u32,
            &bitflipped,
        )
        .unwrap();

        println!("Image saved to `screenshot.png`.");
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

    rl.set_target_fps(60);

    rl.set_window_size(w as i32, h as i32);
    rl.toggle_fullscreen();

    let image = Image::load_image(screenshot_path.as_os_str().to_str().unwrap()).unwrap();
    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

    let mut prev_mouse_pos = rl.get_mouse_position();
    let mut delta_scale = 0.0;
    let zoom_friction = 8.0;
    let zoom_speed = 2.5;

    while !rl.window_should_close() {
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Q)
            || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        {
            let _ = fs::remove_file(screenshot_path);
            break;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
            camera.offset.y -= 10.0;
            camera.target.y -= 10.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
            camera.offset.y += 10.0;
            camera.target.y += 10.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
            camera.offset.x -= 10.0;
            camera.target.x -= 10.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
            camera.offset.x += 10.0;
            camera.target.x += 10.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP) {
            camera.zoom += 0.05;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN) {
            camera.zoom -= 0.05;
        }
        let mouse_delta = rl.get_mouse_wheel_move();
        delta_scale += mouse_delta;

        let delta = rl.get_frame_time();
        camera.zoom = camera.zoom + delta_scale * delta * (zoom_speed * camera.zoom);
        if camera.zoom < 0.01 {
            camera.zoom = 0.01;
        }
        delta_scale -= delta_scale * delta * zoom_friction;
        //let mut new_zoom = camera.zoom + mouse_delta * (0.07f32 * camera.zoom);
        // Capping the zoom so you don't zoom
        // out oo much and get lost
        //if new_zoom <= 0.03 {
        //   new_zoom = 0.03f32;
        //}

        //camera.zoom = new_zoom;

        let mouse_pos = rl.get_mouse_position();
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
        let zoom_amount = format!("{zoom}", zoom = camera.zoom);
        d.draw_text(&zoom_amount, 10, 30, 20, Color::GRAY);

        /*
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::WHITE);
                d.draw_texture(&texture, 0, 0, Color::WHITE);
        */
        //d.draw_text("Press 'ESC' or 'Q' to quit.", 10, 10, 20, Color::BLACK);
    }
}
