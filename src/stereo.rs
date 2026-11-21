use serde::{Deserialize, Serialize};

/// Rectified pinhole stereo, one candidate feature correspondence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StereoInput {
    pub baseline_m: f64,
    pub focal_length_px: f64,
    pub principal_u_px: f64,
    pub left_u_px: f64,
    pub left_v_px: f64,
    pub right_u_px: f64,
    pub right_v_px: f64,
    pub max_epipolar_error_px: f64,
    pub independent_assumed_depth_m: f64,
    pub pixel_sigma_px: f64,
}

impl Default for StereoInput {
    fn default() -> Self {
        Self {
            baseline_m: 0.2,
            focal_length_px: 500.0,
            principal_u_px: 320.0,
            left_u_px: 350.0,
            left_v_px: 240.0,
            right_u_px: 330.0,
            right_v_px: 240.0,
            max_epipolar_error_px: 1.0,
            independent_assumed_depth_m: 3.0,
            pixel_sigma_px: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Reconstruction {
    pub x_in_left_frame_m: f64,
    pub depth_m: f64,
    pub approximate_depth_sigma_m: f64,
    pub left_reprojection_error_px: f64,
    pub right_reprojection_error_px: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StereoResult {
    pub independent_world_x_disagreement_m: f64,
    pub disparity_px: f64,
    pub epipolar_error_px: f64,
    pub positive_disparity: bool,
    pub correspondence_admissible: bool,
    pub joint_reconstruction: Option<Reconstruction>,
    pub interpretation: String,
}

pub fn run(input: StereoInput) -> Result<StereoResult, String> {
    let values = [
        input.baseline_m,
        input.focal_length_px,
        input.principal_u_px,
        input.left_u_px,
        input.left_v_px,
        input.right_u_px,
        input.right_v_px,
        input.max_epipolar_error_px,
        input.independent_assumed_depth_m,
        input.pixel_sigma_px,
    ];
    if values.iter().any(|x| !x.is_finite()) {
        return Err("all stereo parameters must be finite".into());
    }
    if input.baseline_m <= 0.0 || input.focal_length_px <= 0.0 {
        return Err("baseline_m and focal_length_px must be positive".into());
    }
    if input.independent_assumed_depth_m <= 0.0 || input.pixel_sigma_px < 0.0 {
        return Err("assumed depth must be positive and pixel_sigma_px nonnegative".into());
    }
    if input.max_epipolar_error_px < 0.0 {
        return Err("max_epipolar_error_px must be nonnegative".into());
    }

    let z0 = input.independent_assumed_depth_m;
    let independent_left_x =
        (input.left_u_px - input.principal_u_px) * z0 / input.focal_length_px;
    let independent_right_x = input.baseline_m
        + (input.right_u_px - input.principal_u_px) * z0 / input.focal_length_px;
    let disagreement = (independent_left_x - independent_right_x).abs();

    let disparity = input.left_u_px - input.right_u_px;
    let epipolar_error = (input.left_v_px - input.right_v_px).abs();
    let admissible = disparity > 0.0 && epipolar_error <= input.max_epipolar_error_px;
    let joint_reconstruction = if admissible {
        let depth = input.focal_length_px * input.baseline_m / disparity;
        let x = (input.left_u_px - input.principal_u_px) * depth / input.focal_length_px;
        let u_left = input.focal_length_px * x / depth + input.principal_u_px;
        let u_right =
            input.focal_length_px * (x - input.baseline_m) / depth + input.principal_u_px;
        let depth_sigma = input.focal_length_px * input.baseline_m / disparity.powi(2)
            * (2.0_f64).sqrt()
            * input.pixel_sigma_px;
        if !depth.is_finite() || !x.is_finite() || !depth_sigma.is_finite() {
            return Err("stereo geometry exceeds numeric range".into());
        }
        Some(Reconstruction {
            x_in_left_frame_m: x,
            depth_m: depth,
            approximate_depth_sigma_m: depth_sigma,
            left_reprojection_error_px: (u_left - input.left_u_px).abs(),
            right_reprojection_error_px: (u_right - input.right_u_px).abs(),
        })
    } else {
        None
    };

    let interpretation = if disparity <= 0.0 {
        "Rejected: the candidate has non-positive disparity in this rectified forward-facing stereo model."
    } else if !admissible {
        "Rejected: corresponding points violate the rectified epipolar tolerance."
    } else {
        "A common point can explain this candidate under the stated calibration. This alone does not prove feature identity or calibration correctness."
    };
    Ok(StereoResult {
        independent_world_x_disagreement_m: disagreement,
        disparity_px: disparity,
        epipolar_error_px: epipolar_error,
        positive_disparity: disparity > 0.0,
        correspondence_admissible: admissible,
        joint_reconstruction,
        interpretation: interpretation.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_geometry_recovers_known_depth() {
        let r = run(StereoInput::default()).unwrap();
        assert!(r.correspondence_admissible);
        let p = r.joint_reconstruction.unwrap();
        assert!((p.depth_m - 5.0).abs() < 1e-10);
        assert!((p.x_in_left_frame_m - 0.3).abs() < 1e-10);
        assert!(r.independent_world_x_disagreement_m > 0.0);
    }

    #[test]
    fn inconsistent_epipolar_pair_is_not_a_reconstruction() {
        let r = run(StereoInput { right_v_px: 250.0, ..Default::default() }).unwrap();
        assert!(!r.correspondence_admissible);
        assert!(r.joint_reconstruction.is_none());
    }

    #[test]
    fn zero_disparity_is_rejected() {
        let r = run(StereoInput { right_u_px: 350.0, ..Default::default() }).unwrap();
        assert!(r.joint_reconstruction.is_none());
    }
}
