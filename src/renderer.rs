use glam::{Vec3, Vec4};
use crate::{PointCloud, Camera, Point3D, Result, AltostratusError};

/// Core trait for rendering point clouds with different output types
pub trait Renderer {
    /// Output type produced by this renderer (String for ASCII, Image for graphics)
    type Output;

    /// Render a point cloud using the given camera
    ///
    /// # Arguments
    /// * `points` - Point cloud to render
    /// * `camera` - Camera defining the view
    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output>;

    /// Set the viewport size for rendering
    ///
    /// # Arguments
    /// * `width` - Viewport width (pixels for image, characters for ASCII)
    /// * `height` - Viewport height (pixels for image, characters for ASCII)
    fn set_viewport(&mut self, width: u32, height: u32) -> Result<()>;

    /// Get the current viewport size as (width, height)
    fn viewport_size(&self) -> (u32, u32);
}

/// Represents a 2D screen coordinate with depth information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    /// X coordinate in screen space (0 to viewport_width-1)
    pub x: f32,
    /// Y coordinate in screen space (0 to viewport_height-1)  
    pub y: f32,
    /// Depth value (0.0 = near plane, 1.0 = far plane)
    pub depth: f32,
}

impl ScreenPoint {
    /// Creates a new screen point
    ///
    /// # Arguments
    /// * `x` - X screen coordinate
    /// * `y` - Y screen coordinate
    /// * `depth` - Depth value (0.0-1.0)
    pub fn new(x: f32, y: f32, depth: f32) -> Self {
        Self { x, y, depth }
    }

    /// Returns true if this point is within the given viewport bounds
    ///
    /// # Arguments
    /// * `width` - Viewport width
    /// * `height` - Viewport height
    pub fn is_in_bounds(&self, width: u32, height: u32) -> bool {
        self.x >= 0.0 && self.x < width as f32 &&
            self.y >= 0.0 && self.y < height as f32 &&
            self.depth >= 0.0 && self.depth <= 1.0
    }

    /// Returns integer pixel coordinates, clamping to viewport bounds
    ///
    /// # Arguments
    /// * `width` - Viewport width
    /// * `height` - Viewport height
    pub fn to_pixel_coords(&self, width: u32, height: u32) -> (u32, u32) {
        let x = self.x.clamp(0.0, width as f32 - 1.0) as u32;
        let y = self.y.clamp(0.0, height as f32 - 1.0) as u32;
        (x, y)
    }
}

/// 3D to 2D projection utilities
#[derive(Debug)]
pub struct Projector {
    viewport_width: u32,
    viewport_height: u32,
}

impl Projector {
    /// Creates a new projector with the given viewport size
    ///
    /// # Arguments
    /// * `viewport_width` - Width of the viewport
    /// * `viewport_height` - Height of the viewport
    pub fn new(viewport_width: u32, viewport_height: u32) -> Result<Self> {
        if viewport_width == 0 || viewport_height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Viewport dimensions must be positive".to_string()
            ));
        }

        Ok(Self {
            viewport_width,
            viewport_height,
        })
    }

    /// Sets the viewport size
    ///
    /// # Arguments
    /// * `width` - New viewport width
    /// * `height` - New viewport height
    pub fn set_viewport(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Viewport dimensions must be positive".to_string()
            ));
        }

        self.viewport_width = width;
        self.viewport_height = height;
        Ok(())
    }

    /// Gets the current viewport size
    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport_width, self.viewport_height)
    }

    /// Projects a 3D world point to 2D screen coordinates
    ///
    /// # Arguments
    /// * `world_pos` - 3D position in world space
    /// * `camera` - Camera defining the view
    ///
    /// # Returns
    /// * `Some(ScreenPoint)` if the point is visible
    /// * `None` if the point is outside the view frustum
    pub fn project_point(&self, world_pos: Vec3, camera: &Camera) -> Option<ScreenPoint> {
        // Transform to clip space using camera's view-projection matrix
        let view_proj = camera.view_projection_matrix();
        let world_pos_4d = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        let clip_pos = view_proj * world_pos_4d;

        // Perspective divide to get normalized device coordinates (NDC)
        if clip_pos.w <= 0.0 {
            // Point is behind the camera or at infinity
            return None;
        }

        let ndc_x = clip_pos.x / clip_pos.w;
        let ndc_y = clip_pos.y / clip_pos.w;
        let ndc_z = clip_pos.z / clip_pos.w;

        // Check if point is within the normalized device coordinate cube [-1,1]
        if ndc_x < -1.0 || ndc_x > 1.0 || ndc_y < -1.0 || ndc_y > 1.0 || ndc_z < 0.0 || ndc_z > 1.0 {
            return None;
        }

        // Convert NDC to screen coordinates
        // NDC: (-1,-1) = bottom-left, (1,1) = top-right
        // Screen: (0,0) = top-left, (width,height) = bottom-right
        let screen_x = (ndc_x + 1.0) * 0.5 * self.viewport_width as f32;
        let screen_y = (1.0 - ndc_y) * 0.5 * self.viewport_height as f32; // Flip Y axis

        Some(ScreenPoint::new(screen_x, screen_y, ndc_z))
    }

    /// Projects multiple 3D points to screen coordinates, filtering visible ones
    ///
    /// # Arguments
    /// * `world_points` - Iterator of 3D world positions
    /// * `camera` - Camera defining the view
    ///
    /// # Returns
    /// Vector of (original_index, ScreenPoint) pairs for visible points
    pub fn project_points<'a, I>(&self, world_points: I, camera: &Camera) -> Vec<(usize, ScreenPoint)>
    where
        I: Iterator<Item = (usize, Vec3)>,
    {
        world_points
            .filter_map(|(index, pos)| {
                self.project_point(pos, camera).map(|screen_pos| (index, screen_pos))
            })
            .collect()
    }

    /// Projects a point cloud to screen coordinates
    ///
    /// # Arguments
    /// * `points` - Point cloud to project
    /// * `camera` - Camera defining the view
    ///
    /// # Returns
    /// Vector of (Point3D, ScreenPoint) pairs for visible points
    pub fn project_point_cloud(&self, points: &PointCloud, camera: &Camera) -> Vec<(Point3D, ScreenPoint)> {
        points
            .iter()
            .filter_map(|point| {
                self.project_point(point.position, camera)
                    .map(|screen_pos| (*point, screen_pos))
            })
            .collect()
    }
}

/// Frustum culling utilities for performance optimization
pub struct FrustumCuller {
    /// Cached frustum planes for efficient culling
    planes: [Vec4; 6], // Left, Right, Bottom, Top, Near, Far
}

impl FrustumCuller {
    /// Creates a new frustum culler
    pub fn new() -> Self {
        Self {
            planes: [Vec4::ZERO; 6],
        }
    }

    /// Updates the frustum planes from a camera
    ///
    /// # Arguments
    /// * `camera` - Camera to extract frustum from
    pub fn update_from_camera(&mut self, camera: &Camera) {
        let view_proj = camera.view_projection_matrix();

        // Extract frustum planes from view-projection matrix
        // Each plane is represented as Vec4(a, b, c, d) where ax + by + cz + d = 0
        // We need to work with rows, not columns

        let row0 = view_proj.row(0);
        let row1 = view_proj.row(1);
        let row2 = view_proj.row(2);
        let row3 = view_proj.row(3);

        // Left plane: row3 + row0
        self.planes[0] = (row3 + row0).normalize();

        // Right plane: row3 - row0
        self.planes[1] = (row3 - row0).normalize();

        // Bottom plane: row3 + row1
        self.planes[2] = (row3 + row1).normalize();

        // Top plane: row3 - row1
        self.planes[3] = (row3 - row1).normalize();

        // Near plane: row3 + row2
        self.planes[4] = (row3 + row2).normalize();

        // Far plane: row3 - row2
        self.planes[5] = (row3 - row2).normalize();
    }

    /// Tests if a point is inside the frustum
    ///
    /// # Arguments
    /// * `point` - 3D world position to test
    ///
    /// # Returns
    /// `true` if the point is inside the frustum, `false` otherwise
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            let distance = plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w;
            if distance < 0.0 {
                return false; // Point is outside this plane
            }
        }
        true
    }

    /// Filters a slice of points to only those inside the frustum
    ///
    /// # Arguments
    /// * `points` - Slice of 3D points to filter
    ///
    /// # Returns
    /// Vector of points that are inside the frustum
    pub fn cull_points(&self, points: &[Vec3]) -> Vec<Vec3> {
        points
            .iter()
            .filter(|&&point| self.is_point_inside(point))
            .copied()
            .collect()
    }

    /// Filters a point cloud to only visible points
    ///
    /// # Arguments
    /// * `points` - Point cloud to filter
    ///
    /// # Returns
    /// Vector of Point3D that are inside the frustum
    pub fn cull_point_cloud(&self, points: &PointCloud) -> Vec<Point3D> {
        points
            .iter()
            .filter(|point| self.is_point_inside(point.position))
            .copied()
            .collect()
    }
}

impl Default for FrustumCuller {
    fn default() -> Self {
        Self::new()
    }
}

/// Depth buffer for handling point visibility and depth testing
#[derive(Debug, Clone)]
pub struct DepthBuffer {
    /// Depth values for each pixel (0.0 = near, 1.0 = far)
    depths: Vec<f32>,
    width: u32,
    height: u32,
}

impl DepthBuffer {
    /// Creates a new depth buffer with the given dimensions
    ///
    /// # Arguments
    /// * `width` - Buffer width
    /// * `height` - Buffer height
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Depth buffer dimensions must be positive".to_string()
            ));
        }

        let size = (width * height) as usize;
        Ok(Self {
            depths: vec![1.0; size], // Initialize to far plane
            width,
            height,
        })
    }

    /// Clears the depth buffer (sets all depths to far plane)
    pub fn clear(&mut self) {
        self.depths.fill(1.0);
    }

    /// Resizes the depth buffer
    ///
    /// # Arguments
    /// * `width` - New buffer width
    /// * `height` - New buffer height
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Depth buffer dimensions must be positive".to_string()
            ));
        }

        self.width = width;
        self.height = height;
        let new_size = (width * height) as usize;
        self.depths.resize(new_size, 1.0);
        self.clear();
        Ok(())
    }

    /// Tests if a point passes the depth test and updates the buffer if so
    ///
    /// # Arguments
    /// * `x` - X pixel coordinate
    /// * `y` - Y pixel coordinate  
    /// * `depth` - Depth value to test (0.0-1.0)
    ///
    /// # Returns
    /// `true` if the point passes the depth test (is closer), `false` otherwise
    pub fn test_and_update(&mut self, x: u32, y: u32, depth: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        let index = (y * self.width + x) as usize;
        if depth < self.depths[index] {
            self.depths[index] = depth;
            true
        } else {
            false
        }
    }

    /// Gets the depth value at a specific pixel
    ///
    /// # Arguments
    /// * `x` - X pixel coordinate
    /// * `y` - Y pixel coordinate
    pub fn get_depth(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let index = (y * self.width + x) as usize;
        Some(self.depths[index])
    }

    /// Gets the buffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, PointCloud};

    #[test]
    fn test_screen_point_new() {
        let point = ScreenPoint::new(10.0, 20.0, 0.5);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
        assert_eq!(point.depth, 0.5);
    }

    #[test]
    fn test_screen_point_is_in_bounds() {
        let point = ScreenPoint::new(10.0, 20.0, 0.5);
        assert!(point.is_in_bounds(100, 100));
        assert!(!point.is_in_bounds(5, 100)); // x out of bounds
        assert!(!point.is_in_bounds(100, 15)); // y out of bounds

        let point_negative = ScreenPoint::new(-1.0, 20.0, 0.5);
        assert!(!point_negative.is_in_bounds(100, 100));

        let point_depth_out = ScreenPoint::new(10.0, 20.0, 1.5);
        assert!(!point_depth_out.is_in_bounds(100, 100));
    }

    #[test]
    fn test_screen_point_to_pixel_coords() {
        let point = ScreenPoint::new(10.5, 20.7, 0.5);
        let coords = point.to_pixel_coords(100, 100);
        assert_eq!(coords, (10, 20));

        // Test clamping
        let point_out = ScreenPoint::new(-5.0, 150.0, 0.5);
        let coords_clamped = point_out.to_pixel_coords(100, 100);
        assert_eq!(coords_clamped, (0, 99));
    }

    #[test]
    fn test_projector_new() {
        let projector = Projector::new(800, 600).unwrap();
        assert_eq!(projector.viewport_size(), (800, 600));

        // Test invalid dimensions
        assert!(Projector::new(0, 600).is_err());
        assert!(Projector::new(800, 0).is_err());
    }

    #[test]
    fn test_projector_set_viewport() {
        let mut projector = Projector::new(800, 600).unwrap();
        assert!(projector.set_viewport(1024, 768).is_ok());
        assert_eq!(projector.viewport_size(), (1024, 768));

        // Test invalid dimensions
        assert!(projector.set_viewport(0, 768).is_err());
        assert!(projector.set_viewport(1024, 0).is_err());
    }

    #[test]
    fn test_projector_project_point_simple() {
        let projector = Projector::new(100, 100).unwrap();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        // Point at origin should project to center of screen
        let screen_point = projector.project_point(Vec3::ZERO, &camera).unwrap();

        // Should be roughly at center (50, 50) allowing for floating point precision
        assert!((screen_point.x - 50.0).abs() < 1.0);
        assert!((screen_point.y - 50.0).abs() < 1.0);
        assert!(screen_point.depth > 0.0 && screen_point.depth < 1.0);
    }

    #[test]
    fn test_projector_project_point_behind_camera() {
        let projector = Projector::new(100, 100).unwrap();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        // Point behind camera should not be visible
        let screen_point = projector.project_point(Vec3::new(0.0, 0.0, 10.0), &camera);
        assert!(screen_point.is_none());
    }

    #[test]
    fn test_projector_project_points() {
        let projector = Projector::new(100, 100).unwrap();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let world_points = vec![
            Vec3::ZERO,               // Should be visible
            Vec3::new(0.0, 0.0, 10.0), // Behind camera
            Vec3::new(1.0, 0.0, 0.0),  // Should be visible
        ];

        let projected = projector.project_points(
            world_points.iter().enumerate().map(|(i, &pos)| (i, pos)),
            &camera
        );

        // Should have 2 visible points (indices 0 and 2)
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].0, 0); // Index 0
        assert_eq!(projected[1].0, 2); // Index 2
    }

    #[test]
    fn test_projector_project_point_cloud() {
        let projector = Projector::new(100, 100).unwrap();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let mut cloud = PointCloud::new();
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);
        cloud.add_point_coords(0.0, 0.0, 10.0, Color::GREEN); // Behind camera
        cloud.add_point_coords(1.0, 0.0, 0.0, Color::BLUE);

        let projected = projector.project_point_cloud(&cloud, &camera);

        // Should have 2 visible points
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].0.color, Color::RED);
        assert_eq!(projected[1].0.color, Color::BLUE);
    }

    #[test]
    fn test_frustum_culler_new() {
        let culler = FrustumCuller::new();
        // Basic creation test - planes will be initialized but not meaningful until update
        assert_eq!(culler.planes.len(), 6);
    }

    #[test]
    fn test_frustum_culler_update_and_cull() {
        let mut culler = FrustumCuller::new();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        culler.update_from_camera(&camera);

        // Test point culling
        assert!(culler.is_point_inside(Vec3::ZERO)); // Should be visible
        assert!(!culler.is_point_inside(Vec3::new(0.0, 0.0, 10.0))); // Behind camera
    }

    #[test]
    fn test_frustum_culler_cull_points() {
        let mut culler = FrustumCuller::new();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        culler.update_from_camera(&camera);

        let points = vec![
            Vec3::ZERO,               // Should be visible
            Vec3::new(0.0, 0.0, 10.0), // Behind camera
            Vec3::new(1.0, 0.0, 0.0),  // Should be visible
        ];

        let culled = culler.cull_points(&points);
        assert_eq!(culled.len(), 2); // Only 2 should be visible
    }

    #[test]
    fn test_frustum_culler_cull_point_cloud() {
        let mut culler = FrustumCuller::new();
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
        culler.update_from_camera(&camera);

        let mut cloud = PointCloud::new();
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);
        cloud.add_point_coords(0.0, 0.0, 10.0, Color::GREEN); // Behind camera
        cloud.add_point_coords(1.0, 0.0, 0.0, Color::BLUE);

        let culled = culler.cull_point_cloud(&cloud);
        assert_eq!(culled.len(), 2); // Only 2 should be visible
    }

    #[test]
    fn test_depth_buffer_new() {
        let buffer = DepthBuffer::new(100, 100).unwrap();
        assert_eq!(buffer.dimensions(), (100, 100));

        // Test invalid dimensions
        assert!(DepthBuffer::new(0, 100).is_err());
        assert!(DepthBuffer::new(100, 0).is_err());
    }

    #[test]
    fn test_depth_buffer_clear() {
        let mut buffer = DepthBuffer::new(10, 10).unwrap();

        // Set a depth value
        assert!(buffer.test_and_update(5, 5, 0.5));
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 0.5);

        // Clear and check it's reset to far plane
        buffer.clear();
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 1.0);
    }

    #[test]
    fn test_depth_buffer_resize() {
        let mut buffer = DepthBuffer::new(10, 10).unwrap();
        assert!(buffer.resize(20, 20).is_ok());
        assert_eq!(buffer.dimensions(), (20, 20));

        // Test invalid resize
        assert!(buffer.resize(0, 20).is_err());
    }

    #[test]
    fn test_depth_buffer_test_and_update() {
        let mut buffer = DepthBuffer::new(10, 10).unwrap();

        // First point should pass (closer than 1.0)
        assert!(buffer.test_and_update(5, 5, 0.5));
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 0.5);

        // Closer point should pass
        assert!(buffer.test_and_update(5, 5, 0.3));
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 0.3);

        // Farther point should fail
        assert!(!buffer.test_and_update(5, 5, 0.7));
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 0.3); // Unchanged

        // Out of bounds should fail
        assert!(!buffer.test_and_update(15, 5, 0.1));
        assert!(!buffer.test_and_update(5, 15, 0.1));
    }

    #[test]
    fn test_depth_buffer_get_depth() {
        let buffer = DepthBuffer::new(10, 10).unwrap();

        // Valid coordinates should return far plane initially
        assert_eq!(buffer.get_depth(5, 5).unwrap(), 1.0);

        // Invalid coordinates should return None
        assert!(buffer.get_depth(15, 5).is_none());
        assert!(buffer.get_depth(5, 15).is_none());
    }

    #[test]
    fn test_frustum_culler_default() {
        let culler1 = FrustumCuller::default();
        let culler2 = FrustumCuller::new();
        // Both should have the same initial state
        assert_eq!(culler1.planes.len(), culler2.planes.len());
    }
}