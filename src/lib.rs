use glam::Vec3;

// Module exports
pub mod camera;
pub mod renderer;
pub mod image_renderer;
pub mod ascii_renderer;
pub mod axes;

pub use camera::Camera;
pub use renderer::{Renderer, ScreenPoint, Projector, FrustumCuller, DepthBuffer};
pub use image_renderer::{ImageRenderer, AdvancedImageRenderer, PointStyle};
pub use ascii_renderer::{AsciiRenderer, AdvancedAsciiRenderer, CharacterSet};
pub use axes::{Axes, AxesConfig, WithAxes};

/// Simple RGB color representation with 8-bit channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Creates a new color from RGB values
    ///
    /// # Arguments
    /// * `r` - Red channel (0-255)
    /// * `g` - Green channel (0-255) 
    /// * `b` - Blue channel (0-255)
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Creates a new color from RGB values as a tuple
    pub fn from_rgb(rgb: (u8, u8, u8)) -> Self {
        Self::new(rgb.0, rgb.1, rgb.2)
    }

    /// Creates a new color from RGB array
    pub fn from_array(rgb: [u8; 3]) -> Self {
        Self::new(rgb[0], rgb[1], rgb[2])
    }

    /// Returns the color as an RGB tuple
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Returns the color as an RGB array
    pub fn to_array(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// Common color constants
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128 };
}

/// A 3D point with position and color information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    /// 3D position in space
    pub position: Vec3,
    /// Point color
    pub color: Color,
}

impl Point3D {
    /// Creates a new 3D point
    ///
    /// # Arguments
    /// * `position` - 3D position vector
    /// * `color` - Point color
    pub fn new(position: Vec3, color: Color) -> Self {
        Self { position, color }
    }

    /// Creates a new 3D point from coordinates and color
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - 3D coordinates
    /// * `color` - Point color
    pub fn from_coords(x: f32, y: f32, z: f32, color: Color) -> Self {
        Self::new(Vec3::new(x, y, z), color)
    }

    /// Creates a new 3D point from coordinate array and color array
    pub fn from_arrays(coords: [f32; 3], color: [u8; 3]) -> Self {
        Self::new(
            Vec3::from_array(coords),
            Color::from_array(color)
        )
    }

    /// Gets the x coordinate
    pub fn x(&self) -> f32 {
        self.position.x
    }

    /// Gets the y coordinate
    pub fn y(&self) -> f32 {
        self.position.y
    }

    /// Gets the z coordinate
    pub fn z(&self) -> f32 {
        self.position.z
    }
}

/// Container for a collection of 3D points
#[derive(Debug, Clone)]
pub struct PointCloud {
    points: Vec<Point3D>,
    default_color: Color,
}

impl PointCloud {
    /// Creates a new empty point cloud
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            default_color: Color::WHITE,
        }
    }

    /// Creates a new empty point cloud with a specified default color
    ///
    /// # Arguments
    /// * `default_color` - Color to use for points when no color is specified
    pub fn with_default_color(default_color: Color) -> Self {
        Self {
            points: Vec::new(),
            default_color,
        }
    }

    /// Creates a new point cloud with initial capacity
    ///
    /// # Arguments
    /// * `capacity` - Initial capacity for the points vector
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            default_color: Color::WHITE,
        }
    }

    /// Adds a new point to the cloud
    ///
    /// # Arguments
    /// * `point` - The point to add
    pub fn add_point(&mut self, point: Point3D) {
        self.points.push(point);
    }

    /// Adds a new point from position and color
    ///
    /// # Arguments
    /// * `position` - 3D position vector
    /// * `color` - Point color
    pub fn add_point_with_color(&mut self, position: Vec3, color: Color) {
        self.add_point(Point3D::new(position, color));
    }

    /// Adds a new point from coordinates and color
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - 3D coordinates
    /// * `color` - Point color
    pub fn add_point_coords(&mut self, x: f32, y: f32, z: f32, color: Color) {
        self.add_point(Point3D::from_coords(x, y, z, color));
    }

    /// Adds a new point with default color
    ///
    /// # Arguments
    /// * `position` - 3D position vector
    pub fn add_point_default_color(&mut self, position: Vec3) {
        self.add_point(Point3D::new(position, self.default_color));
    }

    /// Adds a new point from coordinates with default color
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - 3D coordinates
    pub fn add_point_coords_default(&mut self, x: f32, y: f32, z: f32) {
        self.add_point_default_color(Vec3::new(x, y, z));
    }

    /// Adds multiple points from a slice
    ///
    /// # Arguments
    /// * `points` - Slice of points to add
    pub fn add_points(&mut self, points: &[Point3D]) {
        self.points.extend_from_slice(points);
    }

    /// Returns the number of points in the cloud
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns true if the point cloud is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Returns an iterator over the points
    pub fn iter(&self) -> std::slice::Iter<Point3D> {
        self.points.iter()
    }

    /// Returns a reference to the points vector
    pub fn points(&self) -> &[Point3D] {
        &self.points
    }

    /// Clears all points from the cloud
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Sets the default color for new points
    ///
    /// # Arguments
    /// * `color` - New default color
    pub fn set_default_color(&mut self, color: Color) {
        self.default_color = color;
    }

    /// Gets the current default color
    pub fn default_color(&self) -> Color {
        self.default_color
    }

    /// Gets the bounding box of all points as (min, max) corners
    pub fn bounding_box(&self) -> Option<(Vec3, Vec3)> {
        if self.points.is_empty() {
            return None;
        }

        let first_pos = self.points[0].position;
        let mut min = first_pos;
        let mut max = first_pos;

        for point in &self.points[1..] {
            min = min.min(point.position);
            max = max.max(point.position);
        }

        Some((min, max))
    }
}

impl Default for PointCloud {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in altostratus operations
#[derive(Debug, Clone, PartialEq)]
pub enum AltostratusError {
    /// Empty point cloud when points are required
    EmptyPointCloud,
    /// Invalid parameter value
    InvalidParameter(String),
    /// Rendering error
    RenderError(String),
}

impl std::fmt::Display for AltostratusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AltostratusError::EmptyPointCloud => write!(f, "Point cloud is empty"),
            AltostratusError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            AltostratusError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for AltostratusError {}

/// Type alias for Result with AltostratusError
pub type Result<T> = std::result::Result<T, AltostratusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_from_rgb() {
        let color = Color::from_rgb((255, 128, 64));
        assert_eq!(color.to_rgb(), (255, 128, 64));
    }

    #[test]
    fn test_color_from_array() {
        let color = Color::from_array([255, 128, 64]);
        assert_eq!(color.to_array(), [255, 128, 64]);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::RED.to_rgb(), (255, 0, 0));
        assert_eq!(Color::GREEN.to_rgb(), (0, 255, 0));
        assert_eq!(Color::BLUE.to_rgb(), (0, 0, 255));
        assert_eq!(Color::WHITE.to_rgb(), (255, 255, 255));
        assert_eq!(Color::BLACK.to_rgb(), (0, 0, 0));
        assert_eq!(Color::GRAY.to_rgb(), (128, 128, 128));
    }

    #[test]
    fn test_point3d_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let color = Color::RED;
        let point = Point3D::new(pos, color);

        assert_eq!(point.position, pos);
        assert_eq!(point.color, color);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.z(), 3.0);
    }

    #[test]
    fn test_point3d_from_coords() {
        let point = Point3D::from_coords(1.0, 2.0, 3.0, Color::BLUE);
        assert_eq!(point.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(point.color, Color::BLUE);
    }

    #[test]
    fn test_point3d_from_arrays() {
        let point = Point3D::from_arrays([1.0, 2.0, 3.0], [255, 0, 0]);
        assert_eq!(point.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(point.color, Color::RED);
    }

    #[test]
    fn test_pointcloud_creation() {
        let cloud = PointCloud::new();
        assert!(cloud.is_empty());
        assert_eq!(cloud.len(), 0);
        assert_eq!(cloud.default_color(), Color::WHITE);
    }

    #[test]
    fn test_pointcloud_with_default_color() {
        let cloud = PointCloud::with_default_color(Color::RED);
        assert_eq!(cloud.default_color(), Color::RED);
    }

    #[test]
    fn test_pointcloud_with_capacity() {
        let cloud = PointCloud::with_capacity(100);
        assert!(cloud.is_empty());
    }

    #[test]
    fn test_pointcloud_add_point() {
        let mut cloud = PointCloud::new();
        let point = Point3D::from_coords(1.0, 2.0, 3.0, Color::RED);

        cloud.add_point(point);
        assert_eq!(cloud.len(), 1);
        assert!(!cloud.is_empty());
        assert_eq!(cloud.points()[0], point);
    }

    #[test]
    fn test_pointcloud_add_point_with_color() {
        let mut cloud = PointCloud::new();
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let color = Color::GREEN;

        cloud.add_point_with_color(pos, color);
        assert_eq!(cloud.len(), 1);

        let added_point = cloud.points()[0];
        assert_eq!(added_point.position, pos);
        assert_eq!(added_point.color, color);
    }

    #[test]
    fn test_pointcloud_add_point_coords() {
        let mut cloud = PointCloud::new();
        cloud.add_point_coords(1.0, 2.0, 3.0, Color::BLUE);

        assert_eq!(cloud.len(), 1);
        let point = cloud.points()[0];
        assert_eq!(point.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(point.color, Color::BLUE);
    }

    #[test]
    fn test_pointcloud_add_point_default_color() {
        let mut cloud = PointCloud::with_default_color(Color::GRAY);
        let pos = Vec3::new(1.0, 2.0, 3.0);

        cloud.add_point_default_color(pos);
        assert_eq!(cloud.len(), 1);

        let point = cloud.points()[0];
        assert_eq!(point.position, pos);
        assert_eq!(point.color, Color::GRAY);
    }

    #[test]
    fn test_pointcloud_add_point_coords_default() {
        let mut cloud = PointCloud::with_default_color(Color::RED);
        cloud.add_point_coords_default(1.0, 2.0, 3.0);

        assert_eq!(cloud.len(), 1);
        let point = cloud.points()[0];
        assert_eq!(point.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(point.color, Color::RED);
    }

    #[test]
    fn test_pointcloud_add_points() {
        let mut cloud = PointCloud::new();
        let points = vec![
            Point3D::from_coords(1.0, 0.0, 0.0, Color::RED),
            Point3D::from_coords(0.0, 1.0, 0.0, Color::GREEN),
            Point3D::from_coords(0.0, 0.0, 1.0, Color::BLUE),
        ];

        cloud.add_points(&points);
        assert_eq!(cloud.len(), 3);
        assert_eq!(cloud.points()[0], points[0]);
        assert_eq!(cloud.points()[1], points[1]);
        assert_eq!(cloud.points()[2], points[2]);
    }

    #[test]
    fn test_pointcloud_clear() {
        let mut cloud = PointCloud::new();
        cloud.add_point_coords(1.0, 2.0, 3.0, Color::RED);
        assert_eq!(cloud.len(), 1);

        cloud.clear();
        assert!(cloud.is_empty());
        assert_eq!(cloud.len(), 0);
    }

    #[test]
    fn test_pointcloud_set_default_color() {
        let mut cloud = PointCloud::new();
        assert_eq!(cloud.default_color(), Color::WHITE);

        cloud.set_default_color(Color::BLUE);
        assert_eq!(cloud.default_color(), Color::BLUE);
    }

    #[test]
    fn test_pointcloud_bounding_box_empty() {
        let cloud = PointCloud::new();
        assert_eq!(cloud.bounding_box(), None);
    }

    #[test]
    fn test_pointcloud_bounding_box_single_point() {
        let mut cloud = PointCloud::new();
        let pos = Vec3::new(1.0, 2.0, 3.0);
        cloud.add_point_with_color(pos, Color::RED);

        let bbox = cloud.bounding_box().unwrap();
        assert_eq!(bbox.0, pos);
        assert_eq!(bbox.1, pos);
    }

    #[test]
    fn test_pointcloud_bounding_box_multiple_points() {
        let mut cloud = PointCloud::new();
        cloud.add_point_coords(-1.0, -2.0, -3.0, Color::RED);
        cloud.add_point_coords(1.0, 2.0, 3.0, Color::GREEN);
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::BLUE);

        let bbox = cloud.bounding_box().unwrap();
        assert_eq!(bbox.0, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.1, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_pointcloud_iter() {
        let mut cloud = PointCloud::new();
        let points = vec![
            Point3D::from_coords(1.0, 0.0, 0.0, Color::RED),
            Point3D::from_coords(0.0, 1.0, 0.0, Color::GREEN),
        ];
        cloud.add_points(&points);

        let collected: Vec<_> = cloud.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(*collected[0], points[0]);
        assert_eq!(*collected[1], points[1]);
    }

    #[test]
    fn test_pointcloud_default() {
        let cloud = PointCloud::default();
        assert!(cloud.is_empty());
        assert_eq!(cloud.default_color(), Color::WHITE);
    }

    #[test]
    fn test_error_display() {
        let err1 = AltostratusError::EmptyPointCloud;
        assert_eq!(err1.to_string(), "Point cloud is empty");

        let err2 = AltostratusError::InvalidParameter("test".to_string());
        assert_eq!(err2.to_string(), "Invalid parameter: test");

        let err3 = AltostratusError::RenderError("render failed".to_string());
        assert_eq!(err3.to_string(), "Render error: render failed");
    }
}