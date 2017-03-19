
use render::*;
use data::*;

use std;

pub struct Game<'a> {
  pub renderer: Renderer<'a>,

  pub in_fwd: bool,
  pub in_back: bool,
  pub in_left: bool,
  pub in_right: bool,
  pub in_turn_left: bool,
  pub in_turn_right: bool,
  pub in_action: bool,

  pub player: Player,
  t: f32,

  walls: Vec<Wall>,
  floors: Vec<Floor>,
  sprites: Vec<Sprite>,
}

impl<'a> Game<'a> {
  pub fn new(width: usize, height: usize, textures: Vec<&[u8]>, level: String) -> Game {
    use serde_json;
    
    use std::fs::File;
    use std::io::prelude::*;

    let mut data = String::new();
    let _ = File::open(level).unwrap().read_to_string(&mut data);

    let root: serde_json::Value = serde_json::from_str(data.as_str()).unwrap();
    let walls : Vec<Wall> = serde_json::from_value(root["walls"].clone()).unwrap();
    let floors : Vec<Floor> = serde_json::from_value(root["floors"].clone()).unwrap();
    let sprites : Vec<Sprite> = serde_json::from_value(root["sprites"].clone()).unwrap();
    
    Game {
      renderer: Renderer::new(width, height, textures),

      in_fwd: false,
      in_back: false,
      in_left: false,
      in_right: false,
      in_turn_left: false,
      in_turn_right: false,
      in_action: false,

      player: Player {
        point: Point3 {
          x: 0.0,
          y: 0.0,
          z: 0.8,
        },
        dir: 0.0,
      },
      t: 0.0,

      sprites: sprites,
      walls: walls,
      floors: floors
    }
  }

  pub fn update(&mut self, dt: f64) {
    let pi = std::f32::consts::PI;

    let dt_ms = dt as f32;
    if self.in_fwd {
      self.player.point.x += f32::cos(self.player.dir) * dt_ms * 2.0;
      self.player.point.y += f32::sin(self.player.dir) * dt_ms * 2.0;
    }
    if self.in_back {
      self.player.point.x += f32::cos(self.player.dir + pi) * dt_ms * 2.0;
      self.player.point.y += f32::sin(self.player.dir + pi) * dt_ms * 2.0;
    }
    if self.in_left {
      self.player.point.x += f32::cos(self.player.dir + pi / 2.0) * dt_ms * 2.0;
      self.player.point.y += f32::sin(self.player.dir + pi / 2.0) * dt_ms * 2.0;
    }
    if self.in_right {
      self.player.point.x += f32::cos(self.player.dir - pi / 2.0) * dt_ms * 2.0;
      self.player.point.y += f32::sin(self.player.dir - pi / 2.0) * dt_ms * 2.0;
    }

    if self.in_turn_left {
      self.player.dir += dt_ms * 2.0;
    }
    if self.in_turn_right {
      self.player.dir -= dt_ms * 2.0;
    }

    self.t += dt_ms;
    self.renderer.t = self.t;
  }

  pub fn draw(&mut self) {
    self.renderer.buffer.clear();
    self.renderer.draw_background(&self.player);

    for w in &self.walls {
      self.renderer.draw_wall(w, &self.player);
    }

    for s in &self.sprites {
      self.renderer.draw_sprite(s, &self.player);
    }

    for f in &self.floors {
      self.renderer.draw_floor(f, &self.player);
    }
  }

  pub fn get_data(&mut self) -> &[u8] {
    &self.renderer.buffer.pixels
  }
}
