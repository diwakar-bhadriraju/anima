//! Pinhole ray rasterizer: casts one ray per retina cell from the body's
//! camera pose, finds the front-most primitive, and produces per-cell
//! luminance + local contrast. Deterministic (analytic ray tests, fixed
//! cell order, no float sorting of scenes).

use crate::world::{Primitive, Vec3, World};

/// Retina grid: 8 azimuth x 6 elevation cells = 48 vision channels.
pub const RETINA_COLS: usize = 8;
pub const RETINA_ROWS: usize = 6;
/// Half-FOV spans (radians), frozen D-70.1.
pub const AZIMUTH_HALF_FOV: f32 = 0.8; // ~45.8 deg
pub const ELEVATION_HALF_FOV: f32 = 0.55; // ~31.5 deg

/// One retina frame: per-cell (row-major, col fastest) luminance in
/// [0,1] plus per-cell local contrast |lum - neighbor mean|.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub luminance: Vec<f32>,
    pub contrast: Vec<f32>,
}

impl Frame {
    pub fn cells(&self) -> usize {
        RETINA_COLS * RETINA_ROWS
    }
}

/// The camera: body pose (position, yaw, pitch) and the deterministic
/// per-cell ray direction table (recomputed once per beat; a pure
/// function of the pose).
#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn from_body(b: &crate::world::Body) -> Self {
        Self { pos: b.pos, yaw: b.yaw, pitch: b.pitch }
    }

    /// Ray origin + unit direction for cell (col, row). Column maps to
    /// azimuth (yaw-offset), row to elevation (pitch-offset).
    pub fn ray(&self, col: usize, row: usize) -> (Vec3, Vec3) {
        let az = (col as f32 / (RETINA_COLS as f32 - 1.0) - 0.5) * 2.0 * AZIMUTH_HALF_FOV;
        let el = (row as f32 / (RETINA_ROWS as f32 - 1.0) - 0.5) * 2.0 * ELEVATION_HALF_FOV;
        // Camera frame: forward = +Z rotated by yaw (XZ) then pitch (Y).
        let (sy, cy) = (self.yaw + az).sin_cos();
        let (sp, cp) = (self.pitch + el).sin_cos();
        (self.pos, Vec3::new(sy * cp, sp, cy * cp))
    }

    /// Rasterize the scene: per-cell front-most primitive -> luminance;
    /// contrast = |lum - mean of orthogonal neighbors| (deterministic
    /// fixed neighbor order, 1-cell halo for edge cells).
    pub fn raster(&self, prims: &[Primitive]) -> Frame {
        let cells = RETINA_COLS * RETINA_ROWS;
        let mut lum = vec![0.0f32; cells];
        let mut hit = vec![false; cells];
        for row in 0..RETINA_ROWS {
            for col in 0..RETINA_COLS {
                let (o, d) = self.ray(col, row);
                let mut best: Option<f32> = None;
                let mut best_lum = 0.0f32;
                for p in prims {
                    if let Some(t) = p.ray_hit(o, d) {
                        if best.map_or(true, |b| t < b) {
                            best = Some(t);
                            best_lum = p.luminance();
                        }
                    }
                }
                let idx = row * RETINA_COLS + col;
                if best.is_some() {
                    lum[idx] = best_lum;
                    hit[idx] = true;
                }
            }
        }
        // Local contrast vs orthogonal neighbors (up/down/left/right,
        // fixed order; edge cells use the existing neighbor or 0.0).
        let mut contrast = vec![0.0f32; cells];
        for row in 0..RETINA_ROWS {
            for col in 0..RETINA_COLS {
                let idx = row * RETINA_COLS + col;
                if !hit[idx] {
                    continue;
                }
                let mut acc = 0.0f32;
                let mut n = 0u32;
                for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (nr, nc) = (row as i32 + dr, col as i32 + dc);
                    if nr >= 0 && nr < RETINA_ROWS as i32 && nc >= 0 && nc < RETINA_COLS as i32 {
                        let nidx = (nr as usize) * RETINA_COLS + nc as usize;
                        if hit[nidx] {
                            acc += (lum[idx] - lum[nidx]).abs();
                            n += 1;
                        }
                    }
                }
                contrast[idx] = if n > 0 { acc / n as f32 } else { 0.0 };
            }
        }
        Frame { luminance: lum, contrast }
    }
}

/// Convenience: rasterize the whole world from its body pose.
pub fn rasterize(world: &World) -> Frame {
    let cam = Camera::from_body(&world.body);
    let mut prims = world.primitives.clone();
    // Depth-sort NOT needed for ray casting (per-cell front-most), but
    // keep a deterministic order-independent call: ray_hit is per-prim.
    prims.sort_by(|a, b| {
        let da = (a.pos.x - cam.pos.x).abs() + (a.pos.y - cam.pos.y).abs() + (a.pos.z - cam.pos.z).abs();
        let db = (b.pos.x - cam.pos.x).abs() + (b.pos.y - cam.pos.y).abs() + (b.pos.z - cam.pos.z).abs();
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });
    cam.raster(&prims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn determinism_frames_same_seed() {
        let a = World::new(11);
        let b = World::new(11);
        let fa = rasterize(&a);
        let fb = rasterize(&b);
        assert_eq!(fa, fb, "frames must be identical for the same seed");
    }

    #[test]
    fn seed_sensitivity_different_frames() {
        let a = rasterize(&World::new(11));
        let b = rasterize(&World::new(12));
        assert_ne!(a, b, "different seeds must rasterize differently");
        let some_hit_a = a.luminance.iter().any(|&l| l > 0.0);
        let some_hit_b = b.luminance.iter().any(|&l| l > 0.0);
        assert!(some_hit_a && some_hit_b, "both scenes must be visible");
    }

    #[test]
    fn frame_shape_and_bounds() {
        let f = rasterize(&World::new(3));
        assert_eq!(f.cells(), 48);
        assert!(f.luminance.iter().all(|&l| (0.0..=1.0).contains(&l)));
        assert!(f.contrast.iter().all(|&c| (0.0..=1.0).contains(&c)));
    }
}