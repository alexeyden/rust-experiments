use data::*;
use buffer::*;
use util::*;

use std::cmp;
use std::mem;

pub struct Renderer<'a> {
  pub textures: Vec<&'a [u8]>,
  pub buffer: Buffer,
  pub width: usize,
  pub height: usize,
  pub t: f32,
  pub ambient: [u8; 3],

  viewport: (f32, f32),
  p_z: f32,
}

impl<'a> Renderer<'a> {
  pub fn new(width: usize, height: usize, textures: Vec<&[u8]>) -> Renderer {
    let width_pow2 = 1 << f32::log2(width as f32).ceil() as usize;
    let height_pow2 = 1 << f32::log2(height as f32).ceil() as usize;

    let view_width = 0.2;

    Renderer {
      width: width,
      height: height,
      textures: textures,
      viewport: (view_width, (height as f32) / (width as f32) * view_width),
      ambient: [00, 40, 00],
      p_z: 0.1,
      t: 0.0,
      buffer: Buffer::new(width_pow2, height_pow2),
    }
  }

  fn project_segment(&self,
                     p1: &Point3,
                     p2: &Point3,
                     player: &Player)
                     -> Option<((isize, f32, f32, f32), (isize, f32, f32, f32))> {
    let len = vec_len(&[p1.x, p1.y], &[p2.x, p2.y]);

    let cos = f32::cos(-player.dir);
    let sin = f32::sin(-player.dir);

    // view space coords
    let mut vx1 = cos * -(p1.y - player.point.y) - sin * (p1.x - player.point.x);
    let mut vz1 = sin * -(p1.y - player.point.y) + cos * (p1.x - player.point.x);
    let mut vx2 = cos * -(p2.y - player.point.y) - sin * (p2.x - player.point.x);
    let mut vz2 = sin * -(p2.y - player.point.y) + cos * (p2.x - player.point.x);

    // texture x coord
    let mut tx1 = 0.0;
    let mut tx2 = len;

    // ensure that 1st point is on the left side
    if vz1 > vz2 {
      mem::swap(&mut vx1, &mut vx2);
      mem::swap(&mut vz1, &mut vz2);
      mem::swap(&mut tx1, &mut tx2);
    }

    // both points are clipped
    if vz1 < self.p_z && vz2 < self.p_z {
      return None;
    }

    // 1st point is clipped
    if vz1 < self.p_z {
      vx1 = vx2 - (vx2 - vx1) * (vz2 - self.p_z) / (vz2 - vz1);
      vz1 = self.p_z;

      let new_len = vec_len(&[vx1, vz1], &[vx2, vz2]);
      tx1 = if tx1 > 0.0 { new_len } else { len - new_len };
    }

    // projection consts
    let a = self.p_z;
    let b = self.viewport.0 / 2.0;
    let c = self.width as f32 / self.viewport.0;

    // project x coords to the screen space
    let sx1 = ((a * vx1 / vz1 + b) * c) as isize;
    let sx2 = ((a * vx2 / vz2 + b) * c) as isize;

    let (p1, p2) = ((sx1, vx1, vz1, tx1), (sx2, vx2, vz2, tx2));

    // restore original order
    let (p1, p2) = if tx1 > tx2 { (p2, p1) } else { (p1, p2) };

    Some((p1, p2))
  }

  pub fn draw_wall(&mut self, w: &Wall, player: &Player) {
    let p1 = Point3 {
      x: w.x0,
      y: w.y0,
      z: w.z,
    };
    let p2 = Point3 {
      x: w.x1,
      y: w.y1,
      z: w.z,
    };

    let p = self.project_segment(&p1, &p2, player);

    let (p1, p2) = match p {
      Some((p1, p2)) => (p1, p2),
      None => return,
    };

    let (p1, p2) = if p1.0 < p2.0 { (p1, p2) } else { (p2, p1) };
    let (sx1, _, vz1, tx1) = p1;
    let (sx2, _, vz2, tx2) = p2;

    let xmin = cmp::max(sx1, 0);
    let xmax = cmp::min(sx2, (self.width - 1) as isize);

    // projection consts
    let a1 = self.p_z * (player.point.z - w.z);
    let a2 = self.p_z * (player.point.z - w.z - w.h);
    let b = self.viewport.1 / 2.0;
    let c = (self.height as f32) / self.viewport.1;

    for x in xmin..xmax {
      // interpolate 1/z and tx/z
      let kx = ((x - sx1) as f32) / ((sx2 - sx1) as f32);
      let vz = (vz1 * vz2) / ((1.0 - kx) * vz2 + kx * vz1);
      let tx = (tx1 / vz1 + (tx2 / vz2 - tx1 / vz1) * kx) * vz;

      let sy1 = ((a1 / vz + b) * c) as isize;
      let sy2 = ((a2 / vz + b) * c) as isize;

      let (sy1, sy2) = (cmp::min(sy1, sy2), cmp::max(sy1, sy2));

      let tx = (tx * 32.0) as usize % 16;

      let ymin = cmp::max(sy1, 0);
      let ymax = cmp::min(sy2, (self.height - 1) as isize);

      for y in ymin..ymax {
        let ky = (y - sy1) as f32 / (sy2 - sy1) as f32;
        let ty = (ky * w.h * 32.0) as usize % 16;

        if self.buffer.depth(x as usize, y as usize) > vz {
          let (r, g, b);
          {
            let tex = self.textures[w.tex];

            r = tex[(ty * 16 + tx) * 3 + 0];
            g = tex[(ty * 16 + tx) * 3 + 1];
            b = tex[(ty * 16 + tx) * 3 + 2];
          }

          let tinted = color_mul(&[r, g, b], &w.color);
          let color = color_mix(&tinted, &self.ambient, fade(vz));

          self.buffer.set_pixel3(x as usize, y as usize, vz, color);
        }
      }
    }
  }

  pub fn draw_sprite(&mut self, sprite: &Sprite, player: &Player) {
    let cos = f32::cos(-player.dir);
    let sin = f32::sin(-player.dir);

    // view space coords
    let vx = cos * -(sprite.point.y - player.point.y) - sin * (sprite.point.x - player.point.x);
    let vz = sin * -(sprite.point.y - player.point.y) + cos * (sprite.point.x - player.point.x);

    // clipped
    if vz <= self.p_z {
      return;
    }

    let ax = self.p_z;
    let bx = self.viewport.0 / 2.0;
    let cx = self.width as f32 / self.viewport.0;

    let ay = self.p_z;
    let by = self.viewport.1 / 2.0;
    let cy = self.height as f32 / self.viewport.1;

    let sx = ((ax * vx / vz + bx) * cx) as isize;
    let sy = ((ay * (player.point.z - sprite.point.z) / vz + by) * cy) as isize;

    let sw = (ax * cx * 1.0 / vz) as isize;

    let sx1 = clamp(sx - sw / 2, 0, (self.width - 1) as isize);
    let sx2 = clamp(sx + sw / 2, 0, (self.width - 1) as isize);
    let sy1 = clamp(sy - sw / 2, 0, (self.height - 1) as isize);
    let sy2 = clamp(sy + sw / 2, 0, (self.height - 1) as isize);

    for x in sx1..sx2 {
      for y in sy1..sy2 {
        let kx = (x - sx + sw / 2) as f32 / sw as f32;
        let ky = (y - sy + sw / 2) as f32 / sw as f32;

        let tx = (kx * 16.0) as usize % 16;
        let ty = (ky * 16.0) as usize % 16;

        if self.buffer.depth(x as usize, y as usize) <= vz {
          continue;
        }

        let (r, g, b);
        {
          let tex = self.textures[sprite.tex];
          r = tex[(ty * 16 + tx) * 3 + 0];
          g = tex[(ty * 16 + tx) * 3 + 1];
          b = tex[(ty * 16 + tx) * 3 + 2];

          if (r == 0) && (g == 0) & (b == 0) {
            continue;
          }
        }

        let tinted = &color_mul(&[r, g, b], &sprite.color);
        let color = color_mix(&tinted, &self.ambient, fade(f32::abs(vz)));

        self.buffer.set_pixel3(x as usize, y as usize, vz, color);
      }
    }
  }

  pub fn draw_background(&mut self, player: &Player) {
    let cos = f32::cos(player.dir);
    let sin = f32::sin(player.dir);

    for sx in 0..self.width {
      for sy in 0..self.height {
        let fsx = (sx as f32) / self.width as f32 * self.viewport.0 - self.viewport.0 / 2.0;
        let fsy = (sy as f32) / self.height as f32 * self.viewport.1 - self.viewport.1 / 2.0;

        let fsy = f32::abs(fsy);

        let vz = self.p_z * player.point.z / fsy;
        let vx = fsx * vz / self.p_z;

        // top half
        if sy < self.height / 2 {
          self.buffer.set_pixel(sx as usize, sy as usize, self.ambient);

          continue;
        }

        // world space
        let x = cos * vz - sin * -vx + player.point.x;
        let z = sin * vz + cos * -vx + player.point.y;

        let tx = (x * 32.0) as usize % 16;
        let ty = (z * 32.0) as usize % 16;

        let (r, g, b);
        {
          let tex = self.textures[0];
          r = tex[(ty * 16 + tx) * 3 + 0];
          g = tex[(ty * 16 + tx) * 3 + 1];
          b = tex[(ty * 16 + tx) * 3 + 2];
        }

        let tinted = color_mul(&[r, g, b], &[100, 255, 100]);
        let color = color_mix(&tinted, &self.ambient, fade(f32::abs(vz)));

        self.buffer.set_pixel3(sx as usize, sy as usize, vz, color);
      }
    }
  }

  fn draw_polygon(&mut self,
                  points: &Vec<(isize, isize)>,
                  height: f32,
                  player: &Player,
                  tex: usize,
                  tint: &[u8; 3]) {
    // index of leftmost point
    let min_index = points.iter()
      .enumerate()
      .min_by_key(|&(_, x)| x.0)
      .unwrap()
      .0;

    // clock- and counterclock- wise iterators
    let mut top = points.iter().cycle().skip(min_index);
    let mut bottom = points.iter()
      .rev()
      .cycle()
      .skip(points.len() - min_index - 1);

    // prev points in top & bottom chain
    let mut ppt = top.next().unwrap();
    let mut ppb = bottom.next().unwrap();

    // current points in top & bottom chain
    let mut pt = top.next().unwrap();
    let mut pb = bottom.next().unwrap();

    let px0 = ppt;
    let x0 = px0.0;

    let px1 = points.iter().max_by_key(|x| x.0).unwrap();
    let x1 = px1.0;

    let mut xt = x0;
    let mut xb = x0;

    let xmin = cmp::max(x0, 0);
    let xmax = cmp::min(x1 + 1, self.width as isize);

    let cos = f32::cos(player.dir);
    let sin = f32::sin(player.dir);

    for x in xmin..xmax {
      // next point in top chain
      if pt.0 < x {
        ppt = pt;
        pt = top.next().unwrap();
        xt = x;
      }
      // next point in bottom chain
      if pb.0 < x {
        ppb = pb;
        pb = bottom.next().unwrap();
        xb = x;
      }

      if pt.0 - ppt.0 == 0 { continue; }
      if pb.0 - ppb.0 == 0 { continue; }
        
      // interpolate y for segments
      let yt = ppt.1 + (x - xt) * (pt.1 - ppt.1) / (pt.0 - ppt.0);
      let yb = ppb.1 + (x - xb) * (pb.1 - ppb.1) / (pb.0 - ppb.0);

      let ymin = cmp::min(yt, yb);
      let ymax = cmp::max(yt, yb);

      let ymin = cmp::max(ymin, 0);
      let ymax = cmp::min(ymax + 1, self.height as isize);

      let fsx = (x as f32) / self.width as f32 * self.viewport.0 - self.viewport.0 / 2.0;

      for y in ymin..ymax {
        let fsy = (y as f32) / self.height as f32 * self.viewport.1 - self.viewport.1 / 2.0;
        let fsy = fsy.abs();

        // to view space
        let vz = self.p_z * (player.point.z - height) / fsy;
        let vx = fsx * vz / self.p_z;

        if self.buffer.depth(x as usize, y as usize) <= vz {
          continue;
        }

        // to world space
        let wx = cos * vz - sin * -vx + player.point.x;
        let wz = sin * vz + cos * -vx + player.point.y;

        let xt = (wx * 32.0).abs() as usize % 16;
        let yt = (wz * 32.0).abs() as usize % 16;

        let (r, g, b);
        {
          let tex = &self.textures[tex];
          let base = yt * 16 + xt;

          r = tex[base * 3 + 0];
          g = tex[base * 3 + 1];
          b = tex[base * 3 + 2];
        };

        let tinted = color_mul(&[r, g, b], tint);
        let color = color_mix(&tinted, &self.ambient, fade(vz));

        self.buffer.set_pixel3(x as usize, y as usize, vz, color);
      }
    }
  }

  pub fn draw_floor(&mut self, floor: &Floor, player: &Player) {
    if floor.height >= player.point.z {
      return;
    }
    
    // zip points to segments
    let seq = floor.points.iter().zip(floor.points
                                        .iter()
                                        .cycle()
                                        .skip(1));

    // project points x coords to screen
    let points: Vec<_> = {
      let projected = seq.map(|(p1, p2)| {
        self.project_segment(&Point3 {
                                x: p1.x,
                                y: p1.y,
                                z: floor.height,
                              },
                             &Point3 {
                                x: p2.x,
                                y: p2.y,
                                z: floor.height,
                              },
                             player)
      });

      // projection consts
      let a = self.p_z * (player.point.z - floor.height);
      let b = self.viewport.1 / 2.0;
      let c = self.height as f32 / self.viewport.1;

      // filter off screen segments and calc ys
      let filter = |seg| if let Some((p0, _)) = seg {
        let (sx0, _, z0, _) = p0;

        let sy0 = ((a / z0 + b) * c) as isize;

        Some((sx0 as isize, sy0))
      } else {
        //FIXME: fix next segment coords if prev. one was thrown out
        None
      };

      projected.filter_map(filter).collect()
    };

    if points.len() < 3 {
      return;
    }

    self.draw_polygon(&points, floor.height, &player, floor.tex, &floor.color);
  }
}
