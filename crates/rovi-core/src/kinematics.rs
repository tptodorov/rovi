//! Mecanum inverse kinematics, the same as ros2_controllers' `mecanum_drive_controller` (Jazzy),
//! with its parameter names (ADR-0006). Wheel order: FL, FR, RR, RL.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// `wheels_radius`, m.
    pub wheels_radius: f64,
    /// `sum_of_robot_center_projection_on_X_Y_axis`, m.
    pub center_projection_sum: f64,
    /// `base_frame_offset` x, y (m) and theta (rad).
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_theta: f64,
    /// Wheel speed (rad/s) that maps to a wheel command of 1.0. A placeholder until M5.
    pub max_wheel_speed: f64,
}

impl Geometry {
    /// The car (M5, 2026-10-17): 60 mm wheels, wheelbase 0.115 m and track 0.140 m, so a centre
    /// projection sum of 0.1275 m. `max_wheel_speed` is the experimental placeholder (M6 O2).
    pub const CAR: Geometry = Geometry {
        wheels_radius: 0.030,
        center_projection_sum: 0.1275,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_theta: 0.0,
        max_wheel_speed: 40.0,
    };
}

/// Wheel speeds in rad/s: FL, FR, RR, RL.
pub fn inverse(g: &Geometry, vx: f64, vy: f64, wz: f64) -> [f64; 4] {
    let (s, c) = (libm::sin(g.offset_theta), libm::cos(g.offset_theta));
    // Base frame to center frame, as in the controller.
    let cx = c * vx - s * vy + g.offset_y * wz;
    let cy = s * vx + c * vy - g.offset_x * wz;
    let l = g.center_projection_sum * wz;
    let k = 1.0 / g.wheels_radius;
    [
        k * (cx - cy - l),
        k * (cx + cy + l),
        k * (cx - cy + l),
        k * (cx + cy - l),
    ]
}

/// Normalises wheel speeds to wheel commands in -1.0..=1.0, saturating. A non-finite speed (an
/// overflow) stays NaN instead of saturating, so motor control refuses it as a fault.
pub fn normalise(g: &Geometry, wheels: [f64; 4]) -> [f32; 4] {
    wheels.map(|w| {
        let n = w / g.max_wheel_speed;
        if n.is_finite() {
            n.clamp(-1.0, 1.0) as f32
        } else {
            f32::NAN
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: Geometry = Geometry {
        wheels_radius: 0.05,
        center_projection_sum: 0.2,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_theta: 0.0,
        max_wheel_speed: 40.0,
    };

    fn close(a: [f64; 4], b: [f64; 4]) {
        for i in 0..4 {
            assert!((a[i] - b[i]).abs() < 1e-9, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn forward_drives_all_wheels_equally() {
        close(inverse(&G, 1.0, 0.0, 0.0), [20.0, 20.0, 20.0, 20.0]);
    }

    #[test]
    fn strafe_left_turns_diagonals() {
        close(inverse(&G, 0.0, 1.0, 0.0), [-20.0, 20.0, -20.0, 20.0]);
    }

    #[test]
    fn rotate_ccw_turns_left_side_back() {
        close(inverse(&G, 0.0, 0.0, 1.0), [-4.0, 4.0, 4.0, -4.0]);
    }

    #[test]
    fn base_frame_offset_translation() {
        // Controller: vx_c = vx + off_y*wz, vy_c = vy - off_x*wz.
        let g = Geometry {
            offset_x: 0.1,
            offset_y: 0.05,
            ..G
        };
        close(inverse(&g, 1.0, 0.0, 1.0), [19.0, 23.0, 27.0, 15.0]);
    }

    #[test]
    fn base_frame_offset_rotation() {
        // A 90 degree offset makes body-forward look like strafe-left.
        let g = Geometry {
            offset_theta: core::f64::consts::FRAC_PI_2,
            ..G
        };
        close(inverse(&g, 1.0, 0.0, 0.0), [-20.0, 20.0, -20.0, 20.0]);
    }

    #[test]
    fn normalise_scales_and_saturates() {
        assert_eq!(
            normalise(&G, [20.0, -10.0, 0.0, 40.0]),
            [0.5, -0.25, 0.0, 1.0]
        );
        assert_eq!(
            normalise(&G, [80.0, -80.0, 0.0, 0.0]),
            [1.0, -1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn a_non_finite_wheel_speed_stays_non_finite_for_motor_fault_handling() {
        let w = normalise(&G, [f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 1.0]);
        assert!(w[0].is_nan() && w[1].is_nan() && w[2].is_nan(), "{w:?}");
        assert_eq!(w[3], (1.0 / G.max_wheel_speed) as f32);
    }

    #[test]
    fn combined_inputs_saturate_each_wheel_independently() {
        // vx 1.5 and vy 1.0: FR and RL would be 1.25, FL and RR 0.25. Each is clamped on its
        // own; the four are not rescaled to keep their ratio.
        let w = normalise(&G, inverse(&G, 1.5, 1.0, 0.0));
        assert_eq!(w, [0.25, 1.0, 0.25, 1.0]);
    }
}
