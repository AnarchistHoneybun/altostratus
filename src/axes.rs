use crate::{Color, PointCloud};
use glam::Vec3;

/// Configuration for 3D coordinate axes
#[derive(Debug, Clone)]
pub struct AxesConfig {
    /// Length of each axis
    pub length: f32,
    /// Color of the X axis
    pub x_color: Color,
    /// Color of the Y axis
    pub y_color: Color,
    /// Color of the Z axis
    pub z_color: Color,
    /// Spacing between tick marks
    pub tick_spacing: f32,
    /// Length of tick marks
    pub tick_length: f32,
    /// Size of arrowheads
    pub arrow_size: f32,
    /// Whether to show tick marks
    pub show_ticks: bool,
    /// Whether to show arrowheads
    pub show_arrows: bool,
    /// Whether to show axis labels (X, Y, Z)
    pub show_labels: bool,
    /// Number of points per unit length for smooth lines
    pub points_per_unit: f32,
}

impl AxesConfig {
    /// Creates a new axes configuration with default values
    pub fn new() -> Self {
        Self {
            length: 5.0,
            x_color: Color::RED,
            y_color: Color::GREEN,
            z_color: Color::BLUE,
            tick_spacing: 1.0,
            tick_length: 0.1,
            arrow_size: 0.2,
            show_ticks: true,
            show_arrows: true,
            show_labels: true,
            points_per_unit: 10.0,
        }
    }

    /// Sets the axis length
    pub fn with_length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Sets the axis colors
    pub fn with_colors(mut self, x_color: Color, y_color: Color, z_color: Color) -> Self {
        self.x_color = x_color;
        self.y_color = y_color;
        self.z_color = z_color;
        self
    }

    /// Sets the tick spacing and length
    pub fn with_ticks(mut self, spacing: f32, length: f32) -> Self {
        self.tick_spacing = spacing;
        self.tick_length = length;
        self
    }

    /// Sets the arrow size
    pub fn with_arrow_size(mut self, size: f32) -> Self {
        self.arrow_size = size;
        self
    }

    /// Controls which features are shown
    pub fn with_features(mut self, ticks: bool, arrows: bool, labels: bool) -> Self {
        self.show_ticks = ticks;
        self.show_arrows = arrows;
        self.show_labels = labels;
        self
    }

    /// Sets the line resolution (points per unit length)
    pub fn with_resolution(mut self, points_per_unit: f32) -> Self {
        self.points_per_unit = points_per_unit;
        self
    }
}

impl Default for AxesConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 3D coordinate axes generator
#[derive(Debug)]
pub struct Axes {
    config: AxesConfig,
}

impl Axes {
    /// Creates new axes with the given configuration
    pub fn new(config: AxesConfig) -> Self {
        Self { config }
    }

    /// Creates axes with default configuration
    pub fn default() -> Self {
        Self::new(AxesConfig::default())
    }

    /// Gets the current configuration
    pub fn config(&self) -> &AxesConfig {
        &self.config
    }

    /// Sets a new configuration
    pub fn set_config(&mut self, config: AxesConfig) {
        self.config = config;
    }

    /// Generates all axis geometry as a point cloud
    pub fn generate_points(&self) -> PointCloud {
        let mut cloud = PointCloud::new();

        // Generate main axis lines
        self.add_axis_line(&mut cloud, Vec3::X, self.config.x_color);
        self.add_axis_line(&mut cloud, Vec3::Y, self.config.y_color);
        self.add_axis_line(&mut cloud, Vec3::Z, self.config.z_color);

        // Generate tick marks
        if self.config.show_ticks {
            self.add_axis_ticks(&mut cloud, Vec3::X, self.config.x_color);
            self.add_axis_ticks(&mut cloud, Vec3::Y, self.config.y_color);
            self.add_axis_ticks(&mut cloud, Vec3::Z, self.config.z_color);
        }

        // Generate arrowheads
        if self.config.show_arrows {
            self.add_axis_arrow(&mut cloud, Vec3::X, self.config.x_color);
            self.add_axis_arrow(&mut cloud, Vec3::Y, self.config.y_color);
            self.add_axis_arrow(&mut cloud, Vec3::Z, self.config.z_color);
        }

        // Generate labels (as simple geometric shapes)
        if self.config.show_labels {
            self.add_axis_labels(&mut cloud);
        }

        cloud
    }

    /// Adds a single axis line from origin to length*direction
    fn add_axis_line(&self, cloud: &mut PointCloud, direction: Vec3, color: Color) {
        let num_points = (self.config.length * self.config.points_per_unit) as usize;

        for i in 0..=num_points {
            let t = i as f32 / num_points as f32;
            let position = direction * (t * self.config.length);
            cloud.add_point_with_color(position, color);
        }
    }

    /// Adds tick marks along an axis
    fn add_axis_ticks(&self, cloud: &mut PointCloud, direction: Vec3, color: Color) {
        if self.config.tick_spacing <= 0.0 {
            return;
        }

        let num_ticks = (self.config.length / self.config.tick_spacing) as usize;

        // Choose perpendicular directions for tick marks
        let (perp1, perp2) = self.get_perpendicular_dirs(direction);

        for i in 1..=num_ticks {
            let position = direction * (i as f32 * self.config.tick_spacing);

            // Add tick marks in both perpendicular directions
            self.add_tick_mark(cloud, position, perp1, color);
            self.add_tick_mark(cloud, position, perp2, color);
        }
    }

    /// Adds a single tick mark
    fn add_tick_mark(&self, cloud: &mut PointCloud, center: Vec3, direction: Vec3, color: Color) {
        let half_length = self.config.tick_length * 0.5;
        let num_points = (self.config.tick_length * self.config.points_per_unit * 2.0) as usize;
        let num_points = num_points.max(3); // Minimum 3 points per tick

        for i in 0..=num_points {
            let t = (i as f32 / num_points as f32) * 2.0 - 1.0; // -1 to 1
            let position = center + direction * (t * half_length);
            cloud.add_point_with_color(position, color);
        }
    }

    /// Adds an arrowhead at the end of an axis
    fn add_axis_arrow(&self, cloud: &mut PointCloud, direction: Vec3, color: Color) {
        let tip_pos = direction * self.config.length;
        let base_pos = tip_pos - direction * self.config.arrow_size;

        // Get perpendicular directions for arrow wings
        let (perp1, perp2) = self.get_perpendicular_dirs(direction);

        // Create arrow wings
        let wing_length = self.config.arrow_size * 0.5;
        let wing1 = base_pos + perp1 * wing_length;
        let wing2 = base_pos - perp1 * wing_length;
        let wing3 = base_pos + perp2 * wing_length;
        let wing4 = base_pos - perp2 * wing_length;

        // Add arrow lines (tip to each wing)
        self.add_line(cloud, tip_pos, wing1, color);
        self.add_line(cloud, tip_pos, wing2, color);
        self.add_line(cloud, tip_pos, wing3, color);
        self.add_line(cloud, tip_pos, wing4, color);

        // Add base circle for better visibility
        self.add_arrow_base(cloud, base_pos, perp1, perp2, wing_length * 0.7, color);
    }

    /// Adds a circular base for the arrow
    fn add_arrow_base(&self, cloud: &mut PointCloud, center: Vec3, perp1: Vec3, perp2: Vec3, radius: f32, color: Color) {
        let num_points = 8; // Octagon approximation

        for i in 0..num_points {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / num_points as f32;
            let position = center + perp1 * (radius * angle.cos()) + perp2 * (radius * angle.sin());
            cloud.add_point_with_color(position, color);
        }
    }

    /// Adds axis labels (X, Y, Z) as simple geometric shapes
    fn add_axis_labels(&self, cloud: &mut PointCloud) {
        let label_offset = self.config.length + self.config.arrow_size + 0.3;
        let label_size = 0.2;

        // X label
        self.add_x_label(cloud, Vec3::X * label_offset, label_size, self.config.x_color);

        // Y label  
        self.add_y_label(cloud, Vec3::Y * label_offset, label_size, self.config.y_color);

        // Z label
        self.add_z_label(cloud, Vec3::Z * label_offset, label_size, self.config.z_color);
    }

    /// Adds an "X" label as crossed lines
    fn add_x_label(&self, cloud: &mut PointCloud, center: Vec3, size: f32, color: Color) {
        let half_size = size * 0.5;

        // First diagonal line
        let p1 = center + Vec3::new(-half_size, -half_size, 0.0);
        let p2 = center + Vec3::new(half_size, half_size, 0.0);
        self.add_line(cloud, p1, p2, color);

        // Second diagonal line
        let p3 = center + Vec3::new(-half_size, half_size, 0.0);
        let p4 = center + Vec3::new(half_size, -half_size, 0.0);
        self.add_line(cloud, p3, p4, color);
    }

    /// Adds a "Y" label as a Y shape
    fn add_y_label(&self, cloud: &mut PointCloud, center: Vec3, size: f32, color: Color) {
        let half_size = size * 0.5;

        // Vertical line (bottom half)
        let p1 = center + Vec3::new(0.0, -half_size, 0.0);
        let p_mid = center;
        self.add_line(cloud, p1, p_mid, color);

        // Left diagonal (top)
        let p2 = center + Vec3::new(-half_size, half_size, 0.0);
        self.add_line(cloud, p2, p_mid, color);

        // Right diagonal (top)
        let p3 = center + Vec3::new(half_size, half_size, 0.0);
        self.add_line(cloud, p3, p_mid, color);
    }

    /// Adds a "Z" label as a Z shape
    fn add_z_label(&self, cloud: &mut PointCloud, center: Vec3, size: f32, color: Color) {
        let half_size = size * 0.5;

        // Top horizontal line
        let p1 = center + Vec3::new(-half_size, half_size, 0.0);
        let p2 = center + Vec3::new(half_size, half_size, 0.0);
        self.add_line(cloud, p1, p2, color);

        // Diagonal line
        let p3 = center + Vec3::new(-half_size, -half_size, 0.0);
        self.add_line(cloud, p2, p3, color);

        // Bottom horizontal line
        let p4 = center + Vec3::new(half_size, -half_size, 0.0);
        self.add_line(cloud, p3, p4, color);
    }

    /// Adds a line between two points
    fn add_line(&self, cloud: &mut PointCloud, start: Vec3, end: Vec3, color: Color) {
        let distance = (end - start).length();
        let num_points = (distance * self.config.points_per_unit * 2.0) as usize;
        let num_points = num_points.max(2); // Minimum 2 points per line

        for i in 0..=num_points {
            let t = i as f32 / num_points as f32;
            let position = start.lerp(end, t);
            cloud.add_point_with_color(position, color);
        }
    }

    /// Gets two perpendicular directions to the given direction
    fn get_perpendicular_dirs(&self, direction: Vec3) -> (Vec3, Vec3) {
        // Find a vector that's not parallel to direction
        let up = if direction.dot(Vec3::Y).abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };

        let perp1 = direction.cross(up).normalize();
        let perp2 = direction.cross(perp1).normalize();

        (perp1, perp2)
    }
}

/// Helper trait to add axes to any renderer
pub trait WithAxes {
    /// Renders the given point cloud with coordinate axes
    fn render_with_axes(&mut self, points: &PointCloud, camera: &crate::Camera, axes_config: &AxesConfig) -> crate::Result<Self::Output>
    where
        Self: crate::Renderer;
}

impl<T: crate::Renderer> WithAxes for T {
    fn render_with_axes(&mut self, points: &PointCloud, camera: &crate::Camera, axes_config: &AxesConfig) -> crate::Result<T::Output> {
        // Generate axes geometry
        let axes = Axes::new(axes_config.clone());
        let axes_points = axes.generate_points();

        // Combine user points with axes points
        let mut combined_cloud = points.clone();
        for point in axes_points.iter() {
            combined_cloud.add_point(*point);
        }

        // Render combined scene
        self.render(&combined_cloud, camera)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axes_config_new() {
        let config = AxesConfig::new();
        assert_eq!(config.length, 5.0);
        assert_eq!(config.x_color, Color::RED);
        assert_eq!(config.y_color, Color::GREEN);
        assert_eq!(config.z_color, Color::BLUE);
        assert!(config.show_ticks);
        assert!(config.show_arrows);
        assert!(config.show_labels);
    }

    #[test]
    fn test_axes_config_builder() {
        let config = AxesConfig::new()
            .with_length(10.0)
            .with_colors(Color::WHITE, Color::WHITE, Color::WHITE)
            .with_ticks(0.5, 0.2)
            .with_arrow_size(0.3)
            .with_features(false, true, false);

        assert_eq!(config.length, 10.0);
        assert_eq!(config.x_color, Color::WHITE);
        assert_eq!(config.tick_spacing, 0.5);
        assert_eq!(config.tick_length, 0.2);
        assert_eq!(config.arrow_size, 0.3);
        assert!(!config.show_ticks);
        assert!(config.show_arrows);
        assert!(!config.show_labels);
    }

    #[test]
    fn test_axes_default() {
        let axes = Axes::default();
        assert_eq!(axes.config().length, 5.0);
    }

    #[test]
    fn test_axes_generate_points() {
        let config = AxesConfig::new()
            .with_length(2.0)
            .with_features(false, false, false); // Only main lines

        let axes = Axes::new(config);
        let cloud = axes.generate_points();

        // Should have points for X, Y, Z axes
        assert!(!cloud.is_empty());

        // Check that we have points of the right colors
        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;

        for point in cloud.iter() {
            if point.color == Color::RED { has_red = true; }
            if point.color == Color::GREEN { has_green = true; }
            if point.color == Color::BLUE { has_blue = true; }
        }

        assert!(has_red, "Should have red X-axis points");
        assert!(has_green, "Should have green Y-axis points");
        assert!(has_blue, "Should have blue Z-axis points");
    }

    #[test]
    fn test_axes_with_ticks() {
        let config = AxesConfig::new()
            .with_length(2.0)
            .with_ticks(1.0, 0.1)
            .with_features(true, false, false); // Only ticks

        let axes = Axes::new(config);
        let cloud = axes.generate_points();

        // Should have more points due to tick marks
        assert!(cloud.len() > 60); // Rough estimate
    }

    #[test]
    fn test_axes_with_arrows() {
        let config = AxesConfig::new()
            .with_length(2.0)
            .with_features(false, true, false); // Only arrows

        let axes = Axes::new(config);
        let cloud = axes.generate_points();

        // Should have points for main lines and arrows
        assert!(cloud.len() > 60); // Main lines + arrow geometry
    }

    #[test]
    fn test_axes_with_labels() {
        let config = AxesConfig::new()
            .with_length(2.0)
            .with_features(false, false, true); // Only labels

        let axes = Axes::new(config);
        let cloud = axes.generate_points();

        // Should have points for main lines and label geometry
        assert!(cloud.len() > 60); // Main lines + label shapes
    }

    #[test]
    fn test_axes_set_config() {
        let mut axes = Axes::default();
        let new_config = AxesConfig::new().with_length(10.0);

        axes.set_config(new_config.clone());
        assert_eq!(axes.config().length, 10.0);
    }

    #[test]
    fn test_get_perpendicular_dirs() {
        let axes = Axes::default();

        // Test with X axis
        let (perp1, perp2) = axes.get_perpendicular_dirs(Vec3::X);
        assert!((perp1.dot(Vec3::X)).abs() < 1e-6);
        assert!((perp2.dot(Vec3::X)).abs() < 1e-6);
        assert!((perp1.dot(perp2)).abs() < 1e-6);

        // Test with Y axis
        let (perp1, perp2) = axes.get_perpendicular_dirs(Vec3::Y);
        assert!((perp1.dot(Vec3::Y)).abs() < 1e-6);
        assert!((perp2.dot(Vec3::Y)).abs() < 1e-6);
        assert!((perp1.dot(perp2)).abs() < 1e-6);
    }

    #[test]
    fn test_axes_config_default() {
        let config1 = AxesConfig::default();
        let config2 = AxesConfig::new();

        assert_eq!(config1.length, config2.length);
        assert_eq!(config1.x_color, config2.x_color);
        assert_eq!(config1.show_ticks, config2.show_ticks);
    }
}