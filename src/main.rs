extern crate sdl2;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::path::Path;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Scancode;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod game;
mod render;
mod data;
mod buffer;
mod util;

use game::*;

pub fn main() {
  let sdl_context = sdl2::init().unwrap();

  let video_subsystem = sdl_context.video().unwrap();
  let mut timer = sdl_context.timer().unwrap();

  let window = video_subsystem.window("Raster", 800, 600)
        .position_centered()
       // .opengl()
        .build()
        .unwrap();

  let mut renderer = window.renderer()
    .accelerated()
    .build()
    .unwrap();

  let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24, 256, 256).unwrap();

  let mut event_pump = sdl_context.event_pump().unwrap();

  let surfaces = vec![Surface::from_file(Path::new("../data/floor.png")).unwrap(),
                      Surface::from_file(Path::new("../data/wall.png")).unwrap(),
                      Surface::from_file(Path::new("../data/sprite.png")).unwrap(),
                      Surface::from_file(Path::new("../data/floor2.png")).unwrap()];

  let mut textures = vec![];
  for s in &surfaces {
    textures.push(s.without_lock().unwrap());
  }

  let mut game = Game::new(200, 150, textures, "../data/map.json".to_owned());

  let mut prev_t = timer.ticks();
  let mut fps_t = 0i64;
  let mut frames = 0;

  'running: loop {
    let now = timer.ticks();
    let dt: i64 = now as i64 - prev_t as i64;
    let sec = dt as f64 / 1_000.0;
    let fps_cap = 30;
    if dt < 1_000 / fps_cap {
      timer.delay((1_000 / fps_cap - dt) as u32);
      continue;
    }
    prev_t = now;

    fps_t += dt;
    frames += 1;
    
    if fps_t > 1000 {
      println!("FPS = {}", frames);
      fps_t -= 1000;
      frames = 0;
    }

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
        _ => {}
      }
    }

    game.in_fwd = event_pump.keyboard_state().is_scancode_pressed(Scancode::W);
    game.in_back = event_pump.keyboard_state().is_scancode_pressed(Scancode::S);
    game.in_left = event_pump.keyboard_state().is_scancode_pressed(Scancode::A);
    game.in_right = event_pump.keyboard_state().is_scancode_pressed(Scancode::D);

    game.in_turn_left = event_pump.keyboard_state().is_scancode_pressed(Scancode::Left);
    game.in_turn_right = event_pump.keyboard_state().is_scancode_pressed(Scancode::Right);

    game.update(sec);
    game.draw();

    texture.update(Some(Rect::new(0,
                                  0,
                                  game.renderer.width as u32,
                                  game.renderer.height as u32)),
                   &game.get_data(),
                   256 * 3)
      .unwrap();
    renderer.copy(&texture,
                  Some(Rect::new(0,
                                 0,
                                 game.renderer.width as u32,
                                 game.renderer.height as u32)),
                  Some(Rect::new(0, 0, 800, 600)))
      .unwrap();
    renderer.present();
  }
}
