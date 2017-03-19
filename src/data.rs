#[derive(Deserialize)]
pub struct Wall {
  pub x0: f32,
  pub y0: f32,
  pub x1: f32,
  pub y1: f32,
  pub z: f32,
  pub h: f32,
  pub tex: usize,
  pub color: [u8; 3],
}

#[derive(Deserialize)]
pub struct Point2 {
  pub x: f32,
  pub y: f32,
}

#[derive(Deserialize)]
pub struct Point3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

pub struct Player {
  pub point: Point3,
  pub dir: f32,
}

#[derive(Deserialize)]
pub struct Floor {
  pub points: Vec<Point2>,
  pub height: f32,
  pub color: [u8; 3],
  pub tex: usize
}

#[derive(Deserialize)]
pub struct Sprite {
  pub point: Point3,
  pub color: [u8; 3],
  pub tex: usize
}
