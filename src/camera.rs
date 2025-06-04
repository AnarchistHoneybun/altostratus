use glam::{Vec3, Mat4};
use crate::{Result, AltostratusError};

/// 3D camera with mutable properties for rendering 3D scenes
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Point the camera is looking at
    pub target: Vec3,
    /// Up vector (typically Vec3::Y)
    pub up: Vec3,
    /// Field of view in radians
    pub fov: f32,
    /// Aspect ratio (width / height)
    pub aspect_ratio: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
}

impl Camera {
    /// Creates a new camera with default settings
    ///
    /// Default setup:
    /// - Position: (0, 0, 5) - 5 units back from origin
    /// - Target: (0, 0, 0) - looking at origin
    /// - Up: (0, 1, 0) - Y-axis up
    /// - FOV: 45 degrees
    /// - Aspect: 1.0 (square)
    /// - Near: 0.1, Far: 100.0
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: std::f32::consts::PI / 4.0, // 45 degrees
            aspect_ratio: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }

    /// Creates a camera with custom position and target
    ///
    /// # Arguments
    /// * `position` - Camera position in world space
    /// * `target` - Point to look at
    pub fn look_at(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: std::f32::consts::PI / 4.0,
            aspect_ratio: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }

    /// Creates a camera with custom field of view and aspect ratio
    ///
    /// # Arguments
    /// * `fov_degrees` - Field of view in degrees
    /// * `aspect_ratio` - Width / height ratio
    pub fn with_perspective(fov_degrees: f32, aspect_ratio: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: fov_degrees.to_radians(),
            aspect_ratio,
            near: 0.1,
            far: 100.0,
        }
    }

    /// Sets the camera position
    ///
    /// # Arguments
    /// * `position` - New camera position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Sets the camera target (look-at point)
    ///
    /// # Arguments
    /// * `target` - New target position
    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
    }

    /// Sets the up vector
    ///
    /// # Arguments
    /// * `up` - New up vector (should be normalized)
    pub fn set_up(&mut self, up: Vec3) {
        self.up = up.normalize();
    }

    /// Sets the field of view in degrees
    ///
    /// # Arguments
    /// * `fov_degrees` - Field of view in degrees (typically 30-90)
    pub fn set_fov_degrees(&mut self, fov_degrees: f32) -> Result<()> {
        if fov_degrees <= 0.0 || fov_degrees >= 180.0 {
            return Err(AltostratusError::InvalidParameter(
                format!("FOV must be between 0 and 180 degrees, got {}", fov_degrees)
            ));
        }
        self.fov = fov_degrees.to_radians();
        Ok(())
    }

    /// Sets the field of view in radians
    ///
    /// # Arguments
    /// * `fov_radians` - Field of view in radians
    pub fn set_fov_radians(&mut self, fov_radians: f32) -> Result<()> {
        if fov_radians <= 0.0 || fov_radians >= std::f32::consts::PI {
            return Err(AltostratusError::InvalidParameter(
                format!("FOV must be between 0 and π radians, got {}", fov_radians)
            ));
        }
        self.fov = fov_radians;
        Ok(())
    }

    /// Sets the aspect ratio (width / height)
    ///
    /// # Arguments
    /// * `aspect_ratio` - New aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Result<()> {
        if aspect_ratio <= 0.0 {
            return Err(AltostratusError::InvalidParameter(
                format!("Aspect ratio must be positive, got {}", aspect_ratio)
            ));
        }
        self.aspect_ratio = aspect_ratio;
        Ok(())
    }

    /// Sets the near and far clipping planes
    ///
    /// # Arguments
    /// * `near` - Near clipping distance (must be positive)
    /// * `far` - Far clipping distance (must be > near)
    pub fn set_clipping_planes(&mut self, near: f32, far: f32) -> Result<()> {
        if near <= 0.0 {
            return Err(AltostratusError::InvalidParameter(
                "Near plane must be positive".to_string()
            ));
        }
        if far <= near {
            return Err(AltostratusError::InvalidParameter(
                "Far plane must be greater than near plane".to_string()
            ));
        }
        self.near = near;
        self.far = far;
        Ok(())
    }

    /// Moves the camera relative to its current position
    ///
    /// # Arguments
    /// * `delta` - Movement vector in world space
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
        self.target += delta;
    }

    /// Moves the camera forward/backward towards/away from the target
    ///
    /// # Arguments
    /// * `distance` - Distance to move (positive = towards target, negative = away from target)
    pub fn move_forward(&mut self, distance: f32) {
        let forward = (self.target - self.position).normalize();
        self.position += forward * distance;
    }

    /// Moves the camera right/left relative to the view direction (orbiting around target)
    ///
    /// # Arguments
    /// * `distance` - Distance to move (positive = right, negative = left)
    pub fn move_right(&mut self, distance: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        self.position += right * distance;
    }

    /// Moves the camera up/down relative to the view direction (orbiting around target)
    ///
    /// # Arguments
    /// * `distance` - Distance to move (positive = up, negative = down)
    pub fn move_up(&mut self, distance: f32) {
        let true_up = self.true_up();
        self.position += true_up * distance;
    }

    /// Zooms the camera by moving towards/away from the target
    ///
    /// # Arguments
    /// * `factor` - Zoom factor (> 1.0 = zoom in, < 1.0 = zoom out)
    pub fn zoom(&mut self, factor: f32) -> Result<()> {
        if factor <= 0.0 {
            return Err(AltostratusError::InvalidParameter(
                "Zoom factor must be positive".to_string()
            ));
        }

        let direction = self.target - self.position;
        let distance = direction.length();

        // Prevent zooming too close or inverting
        let new_distance = distance / factor;
        if new_distance < 0.001 {
            return Err(AltostratusError::InvalidParameter(
                "Cannot zoom closer than 0.001 units".to_string()
            ));
        }

        let new_direction = direction.normalize() * new_distance;
        self.position = self.target - new_direction;
        Ok(())
    }

    /// Orbits the camera around the target point
    ///
    /// # Arguments
    /// * `yaw_delta` - Rotation around the up axis (radians)
    /// * `pitch_delta` - Rotation around the right axis (radians)
    pub fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) -> Result<()> {
        let distance = (self.position - self.target).length();
        if distance < 0.001 {
            return Err(AltostratusError::InvalidParameter(
                "Cannot orbit when camera is at target".to_string()
            ));
        }

        // Convert to spherical coordinates relative to target
        let offset = self.position - self.target;
        let radius = offset.length();

        // Current spherical coordinates
        let mut theta = offset.z.atan2(offset.x); // Yaw (around Y axis)
        let mut phi = (offset.y / radius).asin(); // Pitch (elevation)

        // Apply deltas
        theta += yaw_delta;
        phi += pitch_delta;

        // Clamp pitch to avoid gimbal lock
        phi = phi.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

        // Convert back to Cartesian coordinates
        let new_offset = Vec3::new(
            radius * phi.cos() * theta.cos(),
            radius * phi.sin(),
            radius * phi.cos() * theta.sin(),
        );

        self.position = self.target + new_offset;
        Ok(())
    }

    /// Rotates the camera around its current position (first-person style)
    ///
    /// # Arguments
    /// * `yaw_delta` - Rotation around the up axis (radians)
    /// * `pitch_delta` - Rotation around the right axis (radians)
    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let true_up = right.cross(forward).normalize();

        // Create rotation matrices
        let yaw_rotation = Mat4::from_axis_angle(true_up, yaw_delta);
        let pitch_rotation = Mat4::from_axis_angle(right, pitch_delta);
        let combined_rotation = yaw_rotation * pitch_rotation;

        // Apply rotation to the view direction
        let new_forward = combined_rotation.transform_vector3(forward);
        self.target = self.position + new_forward;
    }

    /// Automatically frames the camera to view the given bounding box
    ///
    /// # Arguments
    /// * `min` - Minimum corner of bounding box
    /// * `max` - Maximum corner of bounding box
    pub fn frame_bounding_box(&mut self, min: Vec3, max: Vec3) -> Result<()> {
        let center = (min + max) * 0.5;
        let size = max - min;
        let max_extent = size.x.max(size.y).max(size.z);

        if max_extent <= 0.0 {
            return Err(AltostratusError::InvalidParameter(
                "Bounding box has no volume".to_string()
            ));
        }

        // Calculate distance to fit the object in view
        let half_fov = self.fov * 0.5;
        let distance = (max_extent * 0.5) / half_fov.tan();

        // Position camera back from center
        let direction = (self.position - self.target).normalize();
        self.target = center;
        self.position = center + direction * (distance + max_extent * 0.1); // Add 10% padding

        Ok(())
    }

    /// Generates the view matrix (world to view space transformation)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Generates the projection matrix (view to clip space transformation)
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    /// Generates the combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Gets the forward direction vector (normalized)
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Gets the right direction vector (normalized)
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }

    /// Gets the true up direction vector (normalized, perpendicular to forward and right)
    pub fn true_up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Gets the distance from camera to target
    pub fn distance_to_target(&self) -> f32 {
        (self.target - self.position).length()
    }

    /// Gets the field of view in degrees
    pub fn fov_degrees(&self) -> f32 {
        self.fov.to_degrees()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_camera_new() {
        let camera = Camera::new();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.up, Vec3::Y);
        assert!((camera.fov - PI / 4.0).abs() < f32::EPSILON);
        assert_eq!(camera.aspect_ratio, 1.0);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 100.0);
    }

    #[test]
    fn test_camera_look_at() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let target = Vec3::new(4.0, 5.0, 6.0);
        let camera = Camera::look_at(pos, target);

        assert_eq!(camera.position, pos);
        assert_eq!(camera.target, target);
        assert_eq!(camera.up, Vec3::Y);
    }

    #[test]
    fn test_camera_with_perspective() {
        let camera = Camera::with_perspective(60.0, 16.0 / 9.0);
        assert!((camera.fov - 60.0_f32.to_radians()).abs() < f32::EPSILON);
        assert!((camera.aspect_ratio - 16.0 / 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_position() {
        let mut camera = Camera::new();
        let new_pos = Vec3::new(1.0, 2.0, 3.0);
        camera.set_position(new_pos);
        assert_eq!(camera.position, new_pos);
    }

    #[test]
    fn test_set_target() {
        let mut camera = Camera::new();
        let new_target = Vec3::new(1.0, 2.0, 3.0);
        camera.set_target(new_target);
        assert_eq!(camera.target, new_target);
    }

    #[test]
    fn test_set_up() {
        let mut camera = Camera::new();
        let new_up = Vec3::new(0.0, 0.0, 1.0);
        camera.set_up(new_up);
        assert_eq!(camera.up, new_up);
    }

    #[test]
    fn test_set_fov_degrees() {
        let mut camera = Camera::new();
        assert!(camera.set_fov_degrees(60.0).is_ok());
        assert!((camera.fov - 60.0_f32.to_radians()).abs() < f32::EPSILON);

        // Test invalid values
        assert!(camera.set_fov_degrees(0.0).is_err());
        assert!(camera.set_fov_degrees(180.0).is_err());
        assert!(camera.set_fov_degrees(-10.0).is_err());
    }

    #[test]
    fn test_set_fov_radians() {
        let mut camera = Camera::new();
        assert!(camera.set_fov_radians(PI / 3.0).is_ok());
        assert!((camera.fov - PI / 3.0).abs() < f32::EPSILON);

        // Test invalid values
        assert!(camera.set_fov_radians(0.0).is_err());
        assert!(camera.set_fov_radians(PI).is_err());
        assert!(camera.set_fov_radians(-1.0).is_err());
    }

    #[test]
    fn test_set_aspect_ratio() {
        let mut camera = Camera::new();
        assert!(camera.set_aspect_ratio(16.0 / 9.0).is_ok());
        assert!((camera.aspect_ratio - 16.0 / 9.0).abs() < f32::EPSILON);

        // Test invalid value
        assert!(camera.set_aspect_ratio(0.0).is_err());
        assert!(camera.set_aspect_ratio(-1.0).is_err());
    }

    #[test]
    fn test_set_clipping_planes() {
        let mut camera = Camera::new();
        assert!(camera.set_clipping_planes(0.5, 200.0).is_ok());
        assert_eq!(camera.near, 0.5);
        assert_eq!(camera.far, 200.0);

        // Test invalid values
        assert!(camera.set_clipping_planes(0.0, 100.0).is_err()); // near <= 0
        assert!(camera.set_clipping_planes(-1.0, 100.0).is_err()); // near < 0
        assert!(camera.set_clipping_planes(10.0, 5.0).is_err()); // far <= near
    }

    #[test]
    fn test_translate() {
        let mut camera = Camera::new();
        let original_pos = camera.position;
        let original_target = camera.target;
        let delta = Vec3::new(1.0, 2.0, 3.0);

        camera.translate(delta);
        assert_eq!(camera.position, original_pos + delta);
        assert_eq!(camera.target, original_target + delta);
    }

    #[test]
    fn test_move_forward() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        let original_distance = camera.distance_to_target();

        camera.move_forward(1.0);
        let new_distance = camera.distance_to_target();

        // Should move closer to target
        assert!(new_distance < original_distance);
        assert!((new_distance - (original_distance - 1.0)).abs() < f32::EPSILON);
        // Target should remain unchanged
        assert_eq!(camera.target, Vec3::ZERO);
    }

    #[test]
    fn test_move_right() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        let original_pos = camera.position;

        camera.move_right(1.0);

        // Should move in the positive X direction for this setup
        assert!(camera.position.x > original_pos.x);
        // Target should remain unchanged
        assert_eq!(camera.target, Vec3::ZERO);
    }

    #[test]
    fn test_move_up() {
        let mut camera = Camera::new();
        let original_pos = camera.position;

        camera.move_up(1.0);

        // For default camera, true_up should be Vec3::Y, so should move up in Y
        assert!((camera.position.y - (original_pos.y + 1.0)).abs() < f32::EPSILON);
        // Target should remain unchanged
        assert_eq!(camera.target, Vec3::ZERO);
    }

    #[test]
    fn test_zoom() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO);
        let original_distance = camera.distance_to_target();

        // Zoom in (factor > 1)
        assert!(camera.zoom(2.0).is_ok());
        let new_distance = camera.distance_to_target();
        assert!((new_distance - original_distance / 2.0).abs() < f32::EPSILON);

        // Test invalid zoom
        assert!(camera.zoom(0.0).is_err());
        assert!(camera.zoom(-1.0).is_err());
    }

    #[test]
    fn test_zoom_too_close() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 0.01), Vec3::ZERO);
        // Should fail when trying to zoom too close
        assert!(camera.zoom(100.0).is_err());
    }

    #[test]
    fn test_orbit() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        let original_distance = camera.distance_to_target();

        assert!(camera.orbit(PI / 4.0, 0.0).is_ok());

        // Distance should remain the same
        let new_distance = camera.distance_to_target();
        assert!((new_distance - original_distance).abs() < f32::EPSILON);

        // Position should have changed
        assert!(camera.position.x != 0.0); // Should have moved in X
    }

    #[test]
    fn test_orbit_at_target() {
        let mut camera = Camera::look_at(Vec3::ZERO, Vec3::ZERO);
        // Should fail when camera is at target
        assert!(camera.orbit(PI / 4.0, 0.0).is_err());
    }

    #[test]
    fn test_rotate() {
        let mut camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        let original_pos = camera.position;

        camera.rotate(PI / 4.0, 0.0);

        // Position should stay the same
        assert_eq!(camera.position, original_pos);
        // Target should have changed
        assert!(camera.target != Vec3::ZERO);
    }

    #[test]
    fn test_frame_bounding_box() {
        let mut camera = Camera::new();
        let min = Vec3::new(-1.0, -1.0, -1.0);
        let max = Vec3::new(1.0, 1.0, 1.0);

        assert!(camera.frame_bounding_box(min, max).is_ok());

        // Target should be at center
        let expected_center = (min + max) * 0.5;
        assert_eq!(camera.target, expected_center);

        // Camera should be positioned to view the box
        assert!(camera.distance_to_target() > 0.0);
    }

    #[test]
    fn test_frame_bounding_box_invalid() {
        let mut camera = Camera::new();
        let point = Vec3::ZERO;
        // Should fail for zero-volume bounding box
        assert!(camera.frame_bounding_box(point, point).is_err());
    }

    #[test]
    fn test_direction_vectors() {
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let forward = camera.forward();
        let right = camera.right();
        let up = camera.true_up();

        // Forward should point towards negative Z
        assert!(forward.z < 0.0);

        // Vectors should be normalized
        assert!((forward.length() - 1.0).abs() < f32::EPSILON);
        assert!((right.length() - 1.0).abs() < f32::EPSILON);
        assert!((up.length() - 1.0).abs() < f32::EPSILON);

        // Vectors should be orthogonal
        assert!(forward.dot(right).abs() < f32::EPSILON);
        assert!(forward.dot(up).abs() < f32::EPSILON);
        assert!(right.dot(up).abs() < f32::EPSILON);
    }

    #[test]
    fn test_distance_to_target() {
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        assert!((camera.distance_to_target() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fov_degrees() {
        let camera = Camera::new();
        assert!((camera.fov_degrees() - 45.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_view_matrix() {
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        let view_matrix = camera.view_matrix();

        // Should be a 4x4 matrix (test basic properties)
        assert!(!view_matrix.is_nan());
        assert!(view_matrix.determinant() != 0.0); // Should be invertible
    }

    #[test]
    fn test_projection_matrix() {
        let camera = Camera::with_perspective(45.0, 16.0 / 9.0);
        let proj_matrix = camera.projection_matrix();

        // Should be a 4x4 matrix (test basic properties)
        assert!(!proj_matrix.is_nan());
    }

    #[test]
    fn test_view_projection_matrix() {
        let camera = Camera::new();
        let vp_matrix = camera.view_projection_matrix();
        let expected = camera.projection_matrix() * camera.view_matrix();

        // Should equal projection * view
        assert_eq!(vp_matrix, expected);
    }

    #[test]
    fn test_default() {
        let camera1 = Camera::default();
        let camera2 = Camera::new();
        assert_eq!(camera1, camera2);
    }
}