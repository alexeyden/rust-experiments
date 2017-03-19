use std;

pub struct Buffer {
  pub pixels: Vec<u8>,
  pub zbuffer: Vec<f32>,
  pub width: usize,
  pub height: usize,
}

impl Buffer {
  pub fn new(width: usize, height: usize) -> Buffer {
    Buffer {
      pixels: vec![0u8; width * height * 3],
      zbuffer: vec![0.0f32; width * height],
      width: width,
      height: height,
    }
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, rgb: [u8; 3]) {
    self.pixels[(x + y * self.width) * 3 + 0] = rgb[0];
    self.pixels[(x + y * self.width) * 3 + 1] = rgb[1];
    self.pixels[(x + y * self.width) * 3 + 2] = rgb[2];
  }

  pub fn set_pixel3(&mut self, x: usize, y: usize, z: f32, rgb: [u8; 3]) {
    self.set_pixel(x, y, rgb);
    self.zbuffer[x + y * self.width] = z;
  }

  pub fn depth(&mut self, x: usize, y: usize) -> f32 {
    self.zbuffer[x + y * self.width]
  }

  pub fn clear(&mut self) {
    for i in 0..self.width * self.height {
      self.zbuffer[i] = std::f32::MAX;
    }
  }
}
