//! Unity math and `UnityEngine.Random` equivalents used by the ported scripts.

use glam::{EulerRot, Mat3, Quat, Vec3};

/// `Quaternion.Euler(x, y, z)` (degrees): Z, then X, then Y.
pub fn euler(degrees: Vec3) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        degrees.y.to_radians(),
        degrees.x.to_radians(),
        degrees.z.to_radians(),
    )
}

/// `Quaternion.AngleAxis(degrees, axis)`.
pub fn angle_axis(degrees: f32, axis: Vec3) -> Quat {
    Quat::from_axis_angle(axis.normalize(), degrees.to_radians())
}

/// `Quaternion.LookRotation(forward, up)`: +Z along `forward`, +Y towards `up`. `None` for a
/// zero `forward` (Unity logs "Look rotation viewing vector is zero" and keeps the rotation).
pub fn look_rotation(forward: Vec3, up: Vec3) -> Option<Quat> {
    let z = forward.normalize_or_zero();
    if z == Vec3::ZERO {
        return None;
    }
    let x = up.cross(z);
    if x.length_squared() < 1e-12 {
        // `up` parallel to `forward`: Unity falls back to the shortest arc from +Z.
        return Some(Quat::from_rotation_arc(Vec3::Z, z));
    }
    let x = x.normalize();
    let y = z.cross(x);
    Some(Quat::from_mat3(&Mat3::from_cols(x, y, z)).normalize())
}

/// `Quaternion.FromToRotation(from, to)`.
pub fn from_to_rotation(from: Vec3, to: Vec3) -> Quat {
    let (a, b) = (from.normalize_or_zero(), to.normalize_or_zero());
    if a == Vec3::ZERO || b == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_rotation_arc(a, b)
}

/// `Mathf.Lerp` (t clamped to 0..1).
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// `Quaternion.Slerp` (t clamped to 0..1), followed by the normalisation `Transform` applies
/// when the result is assigned to a rotation. `b` may be the zero quaternion (a `Quaternion`
/// field that was never assigned): the interpolation then only scales `a`, so the assigned
/// rotation stays `a`.
pub fn slerp_rotation(a: Quat, b: Quat, t: f32) -> Quat {
    let t = t.clamp(0.0, 1.0);
    let mut dot = a.dot(b);
    let b = if dot < 0.0 {
        dot = -dot;
        -b
    } else {
        b
    };
    let q = if dot < 0.95 {
        let angle = dot.acos();
        let inverse_sin = 1.0 / angle.sin();
        let (wa, wb) = ((angle * (1.0 - t)).sin(), (angle * t).sin());
        (a * wa + b * wb) * inverse_sin
    } else {
        a.lerp(b, t)
    };
    if q.length_squared() > 1e-20 {
        q.normalize()
    } else {
        a
    }
}

/// `UnityEngine.Random`: Xorshift128, as Unity's generator. The original seeds it from the
/// clock, so only the distributions matter, not the sequence.
#[derive(Clone, Debug)]
pub struct Random {
    state: [u32; 4],
}

impl Random {
    /// `Random.InitState(seed)`.
    pub fn new(seed: u32) -> Random {
        // Unity expands the seed with the Mersenne Twister initialisation constant.
        let mut state = [seed, 0, 0, 0];
        for i in 1..4 {
            state[i] = 1_812_433_253u32
                .wrapping_mul(state[i - 1] ^ (state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Random { state }
    }

    fn next(&mut self) -> u32 {
        let [x, y, z, w] = self.state;
        let t = x ^ (x << 11);
        let next = w ^ (w >> 19) ^ t ^ (t >> 8);
        self.state = [y, z, w, next];
        next
    }

    /// `Random.value`: 0..=1.
    pub fn value(&mut self) -> f32 {
        (self.next() & 0x007F_FFFF) as f32 / 0x007F_FFFF as f32
    }

    /// `Random.Range(float min, float max)`: min..=max (either order).
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        let t = self.value();
        min + (max - min) * t
    }

    /// `Random.Range(int min, int max)`: min..max (max exclusive).
    pub fn range_int(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next() % (max - min) as u32) as i32
    }

    /// `Random.insideUnitSphere`.
    pub fn inside_unit_sphere(&mut self) -> Vec3 {
        loop {
            let p = Vec3::new(
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
            );
            if p.length_squared() <= 1.0 {
                return p;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_rotation_points_z_forward() {
        let forward = Vec3::new(1.0, 0.5, -2.0);
        let q = look_rotation(forward, Vec3::Y).unwrap();
        assert!((q * Vec3::Z - forward.normalize()).length() < 1e-5);
        assert!((q * Vec3::X).y.abs() < 1e-5, "no roll");
    }

    #[test]
    fn slerp_towards_unassigned_quaternion_keeps_rotation() {
        let a = euler(Vec3::new(10.0, 20.0, 0.0));
        let q = slerp_rotation(a, Quat::from_xyzw(0.0, 0.0, 0.0, 0.0), 0.05);
        assert!((q - a).length() < 1e-5, "{q:?} {a:?}");
    }

    #[test]
    fn random_stays_in_range() {
        let mut random = Random::new(7);
        for _ in 0..1000 {
            let v = random.range(0.0, -1.5);
            assert!((-1.5..=0.0).contains(&v));
            let i = random.range_int(0, 359);
            assert!((0..359).contains(&i));
            assert!(random.inside_unit_sphere().length() <= 1.0);
        }
    }
}
