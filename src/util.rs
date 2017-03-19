use std::cmp;

pub fn color_mul(c1: &[u8; 3], c2: &[u8; 3]) -> [u8; 3] {
  [(((c1[0] as usize) << 8) * ((c2[0] as usize) << 8) >> 24) as u8,
   (((c1[1] as usize) << 8) * ((c2[1] as usize) << 8) >> 24) as u8,
   (((c1[2] as usize) << 8) * ((c2[2] as usize) << 8) >> 24) as u8]
}

pub fn color_mix(c1: &[u8; 3], c2: &[u8; 3], k: f32) -> [u8; 3] {
  [(((c1[0] as f32) * k) + ((c2[0] as f32) * (1.0 - k))).round() as u8,
   (((c1[1] as f32) * k) + ((c2[1] as f32) * (1.0 - k))).round() as u8,
   (((c1[2] as f32) * k) + ((c2[2] as f32) * (1.0 - k))).round() as u8]
}

pub fn vec_len(v1: &[f32; 2], v2: &[f32; 2]) -> f32 {
  f32::sqrt((v2[0] - v1[0]) * (v2[0] - v1[0]) + (v2[1] - v1[1]) * (v2[1] - v1[1]))
}

pub fn clamp<T>(x: T, mn: T, mx: T) -> T where T: cmp::Ord {
  cmp::min(cmp::max(x, mn), mx)
}

pub fn fade(z: f32) -> f32 {
  f32::min(f32::max(0.0, 1.0 / (f32::powf(z * 0.2, 1.2) + 1.0)), 1.0)
}
