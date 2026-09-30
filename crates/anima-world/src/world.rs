//! Deterministic full-3D world: arena, static primitives, and a body
//! driven by rate-proportional motor commands (io::motor convention).
//!
//! Determinism contract (D-70.1): pure sequential f32 fixed-step
//! integration, no parallel reductions, no HashMap iteration, no
//! external physics. Same seed -> bit-identical state after every step.

/// Seconds per physics step (world steps once per beat, and within a beat
/// the scene is static — the retina frames it).
pub const DT: f32 = 0.5;
/// Arena half-extents (x/z) and ceiling height.
pub const ARENA_XZ: f32 = 50.0;
pub const ARENA_Z_MAX: f32 = 30.0;
/// Velocity demand scale: rate_hz on a motor channel -> m/s (or rad/s for
/// turns); linear proportional per the "how much it activates, that much
/// it moves" contract.
pub const K_V: f32 = 0.05;
pub const K_TURN: f32 = 0.02;
/// Clamps.
pub const V_MAX: f32 = 5.0;
pub const TURN_MAX: f32 = 2.0;
/// Per-step exponential velocity friction.
pub const FRICTION: f32 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
}

/// The organism's body: 3D position, heading yaw (radians, about +Y) and
/// pitch (radians, about +X), plus velocities. Full 3D kinematics; gravity
/// OFF in `Body::step` (registered D-70.1; the survival bin adds the
/// environment rule), ground plane z=0 clamps with zero bounce. Energy
/// tank: starts at 100.0, drained by `World::touch_food` (1.0/beat),
/// refilled on food contact; `alive=false` exactly when it hits 0.
#[derive(Debug, Clone)]
pub struct Body {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub yaw_rate: f32,
    pub pitch_rate: f32,
    /// Energy tank (registered: start 100.0, drain 1.0/beat via
    /// `World::touch_food`).
    pub energy: f32,
    /// Dead exactly when energy reaches 0 (registered death rule).
    pub alive: bool,
}

impl Body {
    pub fn new(pos: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw,
            pitch,
            yaw_rate: 0.0,
            pitch_rate: 0.0,
            energy: 100.0,
            alive: true,
        }
    }

    /// Apply one fixed step with the given motor drive (rates_hz per
    /// output channel, linear proportional, clamped; see D-70.1 doc).
    /// Channel map (12): [0]=thrust(fwd), [1]=strafe(right), [2]=lift,
    /// [3]-[4]=yaw L/R, [5]-[6]=pitch U/D, [7]=brake, [8..11]=no-op.
    pub fn step(&mut self, rates: &[f32]) {
        let g = |i: usize| rates.get(i).copied().unwrap_or(0.0).clamp(-150.0, 150.0);
        // Forward direction from yaw (XZ plane), pitch raises the thrust
        // component vertically.
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let fwd = Vec3::new(sy * cp, sp, cy * cp);
        // Right direction (yaw only).
        let right = Vec3::new(cy, 0.0, -sy);
        let thrust = g(0) * K_V;
        let strafe = g(1) * K_V;
        let lift = g(2) * K_V;
        self.vel.x += fwd.x * thrust + right.x * strafe;
        self.vel.y += fwd.y * thrust + lift;
        self.vel.z += fwd.z * thrust + right.z * strafe;
        self.yaw_rate += (g(3) - g(4)) * K_TURN;
        self.pitch_rate += (g(5) - g(6)) * K_TURN;
        // Brake channel damps linear velocity.
        let brake = (g(7) / 150.0).clamp(0.0, 1.0);
        if brake > 0.0 {
            self.vel.x *= 1.0 - 0.5 * brake;
            self.vel.y *= 1.0 - 0.5 * brake;
            self.vel.z *= 1.0 - 0.5 * brake;
        }
        // Friction + clamps.
        self.vel.x *= FRICTION; self.vel.y *= FRICTION; self.vel.z *= FRICTION;
        let speed = (self.vel.x * self.vel.x + self.vel.y * self.vel.y + self.vel.z * self.vel.z).sqrt();
        if speed > V_MAX {
            let k = V_MAX / speed;
            self.vel.x *= k; self.vel.y *= k; self.vel.z *= k;
        }
        self.yaw_rate = self.yaw_rate.clamp(-TURN_MAX, TURN_MAX);
        self.pitch_rate = self.pitch_rate.clamp(-TURN_MAX, TURN_MAX);
        // Integrate.
        self.pos.x += self.vel.x * DT;
        self.pos.y += self.vel.y * DT;
        self.pos.z += self.vel.z * DT;
        self.yaw += self.yaw_rate * DT;
        self.pitch += self.pitch_rate * DT;
        // Arena bounds: clamp position, kill the offending velocity
        // component (registered no-bounce).
        if self.pos.x.abs() > ARENA_XZ { self.pos.x = self.pos.x.clamp(-ARENA_XZ, ARENA_XZ); self.vel.x = 0.0; }
        if self.pos.z.abs() > ARENA_XZ { self.pos.z = self.pos.z.clamp(-ARENA_XZ, ARENA_XZ); self.vel.z = 0.0; }
        if self.pos.y < 0.0 { self.pos.y = 0.0; self.vel.y = 0.0; }
        if self.pos.y > ARENA_Z_MAX { self.pos.y = ARENA_Z_MAX; self.vel.y = 0.0; }
        // Wrap headings.
        while self.yaw > std::f32::consts::PI { self.yaw -= 2.0 * std::f32::consts::PI; }
        while self.yaw < -std::f32::consts::PI { self.yaw += 2.0 * std::f32::consts::PI; }
        while self.pitch > std::f32::consts::PI { self.pitch -= 2.0 * std::f32::consts::PI; }
        while self.pitch < -std::f32::consts::PI { self.pitch += 2.0 * std::f32::consts::PI; }
    }
}

/// Scene shape: ray-testable solid (AABB cube, analytic sphere, or
/// square-based pyramid via Möller-Trumbore triangles). Flat color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Cube { half: f32 },
    Sphere { r: f32 },
    Pyramid { half_base: f32, height: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Primitive {
    pub shape: Shape,
    /// World pose (position; shapes are axis-aligned in yaw for D-70.1).
    pub pos: Vec3,
    /// Flat RGB color in [0,1].
    pub color: [f32; 3],
    /// Food flag: touching within the refill radius refills the body's
    /// energy tank (world-survival goal 1).
    pub food: bool,
}

impl Primitive {
    /// Front-most intersection distance along a ray (origin, unit dir),
    /// or None. Deterministic analytic tests only.
    pub fn ray_hit(&self, o: Vec3, d: Vec3) -> Option<f32> {
        match self.shape {
            Shape::Cube { half } => {
                let (mut tmin, mut tmax) = (f32::NEG_INFINITY, f32::INFINITY);
                for (oi, di, ci) in [(o.x, d.x, self.pos.x), (o.y, d.y, self.pos.y), (o.z, d.z, self.pos.z)] {
                    if di.abs() < 1e-9 {
                        if oi < ci - half || oi > ci + half { return None; }
                    } else {
                        let t1 = (ci - half - oi) / di;
                        let t2 = (ci + half - oi) / di;
                        let (a, b) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
                        tmin = tmin.max(a);
                        tmax = tmax.min(b);
                        if tmin > tmax { return None; }
                    }
                }
                if tmax < 0.0 { None } else { Some(if tmin > 0.0 { tmin } else { tmax }) }
            }
            Shape::Sphere { r } => {
                let lx = o.x - self.pos.x; let ly = o.y - self.pos.y; let lz = o.z - self.pos.z;
                let b = lx * d.x + ly * d.y + lz * d.z;
                let c = lx * lx + ly * ly + lz * lz - r * r;
                let disc = b * b - c;
                if disc < 0.0 { return None; }
                let sq = disc.sqrt();
                let t = if -b - sq > 0.0 { -b - sq } else { -b + sq };
                if t > 0.0 { Some(t) } else { None }
            }
            Shape::Pyramid { half_base, height } => {
                // Apex above the base center; 4 side triangles + base.
                let base = self.pos.y - height * 0.5;
                let top = self.pos.y + height * 0.5;
                let apex = Vec3::new(self.pos.x, top, self.pos.z);
                let corners = [
                    Vec3::new(self.pos.x - half_base, base, self.pos.z - half_base),
                    Vec3::new(self.pos.x + half_base, base, self.pos.z - half_base),
                    Vec3::new(self.pos.x + half_base, base, self.pos.z + half_base),
                    Vec3::new(self.pos.x - half_base, base, self.pos.z + half_base),
                ];
                let mut best: Option<f32> = None;
                for (a, b, c) in [
                    (apex, corners[0], corners[1]),
                    (apex, corners[1], corners[2]),
                    (apex, corners[2], corners[3]),
                    (apex, corners[3], corners[0]),
                    (corners[0], corners[1], corners[3]),
                    (corners[1], corners[2], corners[3]),
                ] {
                    if let Some(t) = tri_hit(o, d, a, b, c) {
                        best = Some(match best { Some(p) => p.min(t), None => t });
                    }
                }
                best
            }
        }
    }

    /// Luminance (fixed weights, deterministic).
    pub fn luminance(&self) -> f32 {
        self.color[0] * 0.299 + self.color[1] * 0.587 + self.color[2] * 0.114
    }
}

/// Möller-Trumbore ray-triangle intersection (deterministic).
fn tri_hit(o: Vec3, d: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<f32> {
    let e1 = Vec3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
    let e2 = Vec3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-9 { return None; }
    let inv = 1.0 / det;
    let s = Vec3::new(o.x - v0.x, o.y - v0.y, o.z - v0.z);
    let u = dot(s, p) * inv;
    if u < 0.0 || u > 1.0 { return None; }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 { return None; }
    let t = dot(e2, q) * inv;
    if t > 0.0 { Some(t) } else { None }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// The world: arena + body + static scene. Seeded builder.
#[derive(Debug, Clone)]
pub struct World {
    pub body: Body,
    pub primitives: Vec<Primitive>,
}

impl World {
    /// Build a deterministic scene from a seed: 5 primitives at hashed
    /// poses within the arena, body at the arena center facing +Z.
    pub fn new(seed: u64) -> Self {
        let mut rng = {
            use rand::SeedableRng;
            let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
            rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(z)
        };
        use rand::Rng;
        let mut primitives = Vec::new();
        // (shape, color, food): the green pyramid is THE food source.
        let defs: [(Shape, [f32; 3], bool); 5] = [
            (Shape::Cube { half: 3.0 }, [0.85, 0.15, 0.15], false),
            (Shape::Sphere { r: 3.0 }, [0.15, 0.35, 0.85], false),
            (Shape::Pyramid { half_base: 3.0, height: 6.0 }, [0.15, 0.75, 0.25], true),
            (Shape::Cube { half: 2.0 }, [0.9, 0.9, 0.9], false),
            (Shape::Pyramid { half_base: 2.5, height: 5.0 }, [0.9, 0.85, 0.2], false),
        ];
        for (shape, color, food) in defs {
            // Deterministic placement INSIDE the camera's forward cone
            // (the D-70.1 body starts at the origin facing +Z): z in
            // [5, 40], x within +/-0.9*z (azimuth half-FOV 0.8 rad),
            // y near the camera height (2..4, elevation half-FOV 0.55).
            let z = 5.0 + rng.gen::<f32>() * (ARENA_XZ * 0.8 - 5.0);
            let x = (rng.gen::<f32>() - 0.5) * 2.0 * (z * 0.9);
            let y = 2.0 + rng.gen::<f32>() * 2.0;
            primitives.push(Primitive { shape, pos: Vec3::new(x, y, z), color, food });
        }
        World { body: Body::new(Vec3::new(0.0, 1.5, 0.0), 0.0, 0.0), primitives }
    }

    /// One world step: body responds to the motor drive. Scene static.
    pub fn step(&mut self, rates: &[f32]) {
        self.body.step(rates);
    }

    /// Energy bookkeeping, called once per beat AFTER `step` (refill then
    /// drain). Registered constants (NOT tuned on outcomes): touch radius
    /// 6.0 around the food primitive, refill +30.0, drain 1.0/beat, death
    /// exactly when the tank reaches 0 (clamped, then `alive=false`; a
    /// dead body never drains again). Deterministic: fixed iteration
    /// order over primitives, pure f32 math.
    pub fn touch_food(&mut self) {
        if !self.body.alive {
            return;
        }
        for p in &self.primitives {
            if p.food {
                let dx = p.pos.x - self.body.pos.x;
                let dy = p.pos.y - self.body.pos.y;
                let dz = p.pos.z - self.body.pos.z;
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 < 6.0 * 6.0 {
                    self.body.energy += 30.0;
                    break;
                }
            }
        }
        self.body.energy -= 1.0;
        if self.body.energy <= 0.0 {
            self.body.energy = 0.0;
            self.body.alive = false;
        }
    }
}

/// Motor drive from a 12-dim spike-count output vector — the same linear
/// proportional contract as `io::motor` (rate Hz per output neuron).
pub struct MotorDrive {
    pub rates_hz: Vec<f32>,
}

impl MotorDrive {
    pub fn from_counts(out: &[f32]) -> Self {
        MotorDrive {
            rates_hz: out.iter().map(|x| x * 1000.0 / 500.0).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_world_same_seed() {
        let mut a = World::new(7);
        let mut b = World::new(7);
        let rates: Vec<f32> = (0..12).map(|i| (i as f32) * 7.0).collect();
        for _ in 0..100 {
            a.step(&rates);
            b.step(&rates);
            assert_eq!(a.body.pos, b.body.pos, "position diverged");
            assert_eq!(a.body.yaw, b.body.yaw, "yaw diverged");
            assert_eq!(a.body.pitch, b.body.pitch, "pitch diverged");
            assert_eq!(a.body.vel, b.body.vel, "velocity diverged");
        }
    }

    #[test]
    fn determinism_world_different_seeds_diverge() {
        let mut a = World::new(7);
        let mut b = World::new(8);
        let rates: Vec<f32> = (0..12).map(|i| (i as f32) * 7.0).collect();
        for _ in 0..100 {
            a.step(&rates);
            b.step(&rates);
        }
        // D-70.1: static primitives do not couple to the body, so the
        // body trajectory is seed-independent BY DESIGN; divergence
        // enters via the scene (and, in D-70.2, vision-driven steering).
        assert_ne!(a.primitives[0].pos, b.primitives[0].pos, "scene must differ by seed");
        assert_eq!(a.body.pos, b.body.pos, "body trajectory is seed-independent in D-70.1");
    }

    #[test]
    fn body_thrust_moves_forward() {
        let mut b = Body::new(Vec3::ZERO, 0.0, 0.0);
        // Pure thrust on channel 0 at high rate: forward is +Z when yaw=0.
        let mut rates = vec![0.0f32; 12];
        rates[0] = 100.0;
        for _ in 0..20 {
            b.step(&rates);
        }
        assert!(b.pos.z > 1.0, "thrust must move +Z, got z={}", b.pos.z);
        assert!(b.pos.x.abs() < 1e-3, "no strafe, got x={}", b.pos.x);
    }

    #[test]
    fn bounds_clamp_no_escape() {
        let mut b = Body::new(Vec3::new(0.0, 0.0, 0.0), 0.0, 0.0);
        let mut rates = vec![0.0f32; 12];
        rates[0] = 150.0;
        for _ in 0..1000 {
            b.step(&rates);
        }
        assert!(b.pos.z <= ARENA_XZ + 1e-4, "escaped arena z={}", b.pos.z);
        assert!(b.pos.y >= -1e-4, "below ground y={}", b.pos.y);
    }

    #[test]
    fn fresh_body_starts_full_energy() {
        let w = World::new(7);
        assert_eq!(w.body.energy, 100.0, "fresh body starts with a full tank");
        assert!(w.body.alive, "fresh body is alive");
        // Exactly one food primitive, and it is the green pyramid.
        let foods: Vec<&Primitive> = w.primitives.iter().filter(|p| p.food).collect();
        assert_eq!(foods.len(), 1, "scene has exactly one food source");
        assert_eq!(foods[0].color, [0.15, 0.75, 0.25], "food is the green pyramid");
    }

    #[test]
    fn food_refill_and_exact_death() {
        // Refill: a body spawned adjacent to the food primitive gains
        // +30 on the first touch (then drains 1.0) -> tank > 100.
        let mut a = World::new(7);
        let food = a.primitives.iter().find(|p| p.food).expect("scene has food");
        a.body.pos = Vec3::new(food.pos.x + 1.0, food.pos.y, food.pos.z);
        a.touch_food();
        assert!(a.body.energy > 100.0, "touching food must refill, got {}", a.body.energy);
        assert!(a.body.alive);
        // Death: empty arena -> 1.0/beat drain from 100.0.
        let mut b = World::new(7);
        b.primitives.clear();
        for _ in 0..99 {
            b.touch_food();
        }
        assert!(b.body.alive, "must survive 99 beats (100 -> 1)");
        assert_eq!(b.body.energy, 1.0);
        b.touch_food();
        assert!(!b.body.alive, "must die exactly when energy hits 0");
        assert_eq!(b.body.energy, 0.0, "tank clamps at 0");
        // A dead body never drains again (early return).
        b.touch_food();
        assert_eq!(b.body.energy, 0.0);
    }
}