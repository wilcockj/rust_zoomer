// Add correct pan and zoom controls
// maybe keep screenshot in memory
extern crate raylib;
extern crate repng;
extern crate scrap;

use raylib::prelude::*;

use scrap::{Capturer, Display};
use std::fs::File;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1920, 1080)
        .title("Rust Screenshot")
        .build();

    let mut camera = Camera2D {
        target: Vector2 {
            x: 1920.0 / 2.0,
            y: 1080.0 / 2.0,
        },
        offset: Vector2 {
            x: 1920.0 / 2.0,
            y: 1080.0 / 2.0,
        },
        rotation: 0.0,
        zoom: 1.0,
    };

    rl.set_target_fps(60);

    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

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
            File::create("screenshot.png").unwrap(),
            w as u32,
            h as u32,
            &bitflipped,
        )
        .unwrap();

        println!("Image saved to `screenshot.png`.");
        break;
    }

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();

    rl.set_window_size(width as i32, height as i32);
    rl.toggle_fullscreen();

    let image = Image::load_image("screenshot.png").unwrap();
    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

    let mut prev_mouse_pos = rl.get_mouse_position();

    while !rl.window_should_close() {
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Q)
            || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        {
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
