use image::{RgbImage, Rgb};
use crate::{Renderer, PointCloud, Camera, Color, Result, AltostratusError, Projector, DepthBuffer};

/// Image renderer that outputs PNG images
#[derive(Debug)]
pub struct ImageRenderer {
    /// Image width in pixels
    width: u32,
    /// Image height in pixels
    height: u32,
    /// Background color for the image
    background_color: Color,
    /// Default point size in pixels
    point_size: f32,
    /// 3D to 2D projector
    projector: Projector,
    /// Depth buffer for proper point ordering
    depth_buffer: DepthBuffer,
}

impl ImageRenderer {
    /// Creates a new image renderer with the given dimensions
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let projector = Projector::new(width, height)?;
        let depth_buffer = DepthBuffer::new(width, height)?;

        Ok(Self {
            width,
            height,
            background_color: Color::BLACK,
            point_size: 2.0,
            projector,
            depth_buffer,
        })
    }

    /// Creates a new image renderer with custom background color
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `background_color` - Background color for the image
    pub fn with_background(width: u32, height: u32, background_color: Color) -> Result<Self> {
        let mut renderer = Self::new(width, height)?;
        renderer.background_color = background_color;
        Ok(renderer)
    }

    /// Sets the background color
    ///
    /// # Arguments
    /// * `color` - New background color
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Gets the current background color
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Sets the default point size in pixels
    ///
    /// # Arguments
    /// * `size` - Point size in pixels (must be positive)
    pub fn set_point_size(&mut self, size: f32) -> Result<()> {
        if size <= 0.0 {
            return Err(AltostratusError::InvalidParameter(
                "Point size must be positive".to_string()
            ));
        }
        self.point_size = size;
        Ok(())
    }

    /// Gets the current default point size
    pub fn point_size(&self) -> f32 {
        self.point_size
    }

    /// Draws a point on the image as a filled circle
    ///
    /// # Arguments
    /// * `image` - Mutable reference to the image buffer
    /// * `x` - X center coordinate
    /// * `y` - Y center coordinate
    /// * `size` - Circle radius in pixels
    /// * `color` - Point color
    fn draw_point(&self, image: &mut RgbImage, x: f32, y: f32, size: f32, color: Color) {
        let radius = size.max(1.0);
        let center_x = x as i32;
        let center_y = y as i32;
        let radius_int = radius.ceil() as i32;

        // Draw filled circle using simple distance check
        for dy in -radius_int..=radius_int {
            for dx in -radius_int..=radius_int {
                let pixel_x = center_x + dx;
                let pixel_y = center_y + dy;

                // Check bounds
                if pixel_x < 0 || pixel_y < 0 ||
                    pixel_x >= self.width as i32 || pixel_y >= self.height as i32 {
                    continue;
                }

                // Check if pixel is inside circle
                let distance_sq = (dx * dx + dy * dy) as f32;
                if distance_sq <= radius * radius {
                    let rgb = Rgb([color.r, color.g, color.b]);
                    image.put_pixel(pixel_x as u32, pixel_y as u32, rgb);
                }
            }
        }
    }

    /// Draws a point as a filled square (alternative to circle)
    ///
    /// # Arguments
    /// * `image` - Mutable reference to the image buffer
    /// * `x` - X center coordinate
    /// * `y` - Y center coordinate
    /// * `size` - Square half-width in pixels
    /// * `color` - Point color
    fn draw_point_square(&self, image: &mut RgbImage, x: f32, y: f32, size: f32, color: Color) {
        let half_size = size.max(1.0);
        let center_x = x as i32;
        let center_y = y as i32;
        let half_size_int = half_size.ceil() as i32;

        // Draw filled square
        for dy in -half_size_int..=half_size_int {
            for dx in -half_size_int..=half_size_int {
                let pixel_x = center_x + dx;
                let pixel_y = center_y + dy;

                // Check bounds
                if pixel_x < 0 || pixel_y < 0 ||
                    pixel_x >= self.width as i32 || pixel_y >= self.height as i32 {
                    continue;
                }

                let rgb = Rgb([color.r, color.g, color.b]);
                image.put_pixel(pixel_x as u32, pixel_y as u32, rgb);
            }
        }
    }

    /// Draws a single pixel point (fastest option)
    ///
    /// # Arguments
    /// * `image` - Mutable reference to the image buffer
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `color` - Point color
    fn draw_point_pixel(&self, image: &mut RgbImage, x: f32, y: f32, color: Color) {
        let pixel_x = x.round() as u32;
        let pixel_y = y.round() as u32;

        if pixel_x < self.width && pixel_y < self.height {
            let rgb = Rgb([color.r, color.g, color.b]);
            image.put_pixel(pixel_x, pixel_y, rgb);
        }
    }
}

impl Renderer for ImageRenderer {
    type Output = RgbImage;

    /// Renders a point cloud to an RGB image
    ///
    /// # Arguments
    /// * `points` - Point cloud to render
    /// * `camera` - Camera defining the view
    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output> {
        if points.is_empty() {
            // Return empty image with background color
            let mut image = RgbImage::new(self.width, self.height);
            let bg_rgb = Rgb([self.background_color.r, self.background_color.g, self.background_color.b]);
            for pixel in image.pixels_mut() {
                *pixel = bg_rgb;
            }
            return Ok(image);
        }

        // Update camera's aspect ratio to match our image dimensions
        let mut render_camera = camera.clone();
        let aspect_ratio = self.width as f32 / self.height as f32;
        render_camera.set_aspect_ratio(aspect_ratio)?;

        // Project all points to screen coordinates
        let projected_points = self.projector.project_point_cloud(points, &render_camera);

        if projected_points.is_empty() {
            // No visible points - return background
            let mut image = RgbImage::new(self.width, self.height);
            let bg_rgb = Rgb([self.background_color.r, self.background_color.g, self.background_color.b]);
            for pixel in image.pixels_mut() {
                *pixel = bg_rgb;
            }
            return Ok(image);
        }

        // Create image with background color
        let mut image = RgbImage::new(self.width, self.height);
        let bg_rgb = Rgb([self.background_color.r, self.background_color.g, self.background_color.b]);
        for pixel in image.pixels_mut() {
            *pixel = bg_rgb;
        }

        // Clear depth buffer
        self.depth_buffer.clear();

        // Sort points by depth (back to front for proper rendering)
        let mut sorted_points = projected_points;
        sorted_points.sort_by(|a, b| b.1.depth.partial_cmp(&a.1.depth).unwrap_or(std::cmp::Ordering::Equal));

        // Draw points
        for (point3d, screen_point) in sorted_points {
            let (pixel_x, pixel_y) = screen_point.to_pixel_coords(self.width, self.height);

            // Depth test
            if self.depth_buffer.test_and_update(pixel_x, pixel_y, screen_point.depth) {
                // Choose drawing method based on point size
                if self.point_size <= 1.0 {
                    self.draw_point_pixel(&mut image, screen_point.x, screen_point.y, point3d.color);
                } else if self.point_size <= 3.0 {
                    self.draw_point_square(&mut image, screen_point.x, screen_point.y, self.point_size, point3d.color);
                } else {
                    self.draw_point(&mut image, screen_point.x, screen_point.y, self.point_size, point3d.color);
                }
            }
        }

        Ok(image)
    }

    /// Sets the viewport size (image dimensions)
    ///
    /// # Arguments
    /// * `width` - New image width in pixels
    /// * `height` - New image height in pixels
    fn set_viewport(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Image dimensions must be positive".to_string()
            ));
        }

        self.width = width;
        self.height = height;
        self.projector.set_viewport(width, height)?;
        self.depth_buffer.resize(width, height)?;

        Ok(())
    }

    /// Gets the current viewport size
    fn viewport_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Point drawing styles for different visual effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointStyle {
    /// Single pixel points (fastest)
    Pixel,
    /// Square points with given size
    Square,
    /// Circular points with given radius (default)
    Circle,
}

/// Extended image renderer with more rendering options
#[derive(Debug)]
pub struct AdvancedImageRenderer {
    base: ImageRenderer,
    point_style: PointStyle,
    enable_antialiasing: bool,
}

impl AdvancedImageRenderer {
    /// Creates a new advanced image renderer
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            base: ImageRenderer::new(width, height)?,
            point_style: PointStyle::Circle,
            enable_antialiasing: false,
        })
    }

    /// Sets the point drawing style
    ///
    /// # Arguments
    /// * `style` - Point drawing style
    pub fn set_point_style(&mut self, style: PointStyle) {
        self.point_style = style;
    }

    /// Gets the current point style
    pub fn point_style(&self) -> PointStyle {
        self.point_style
    }

    /// Enables or disables antialiasing (not implemented yet)
    ///
    /// # Arguments
    /// * `enable` - Whether to enable antialiasing
    pub fn set_antialiasing(&mut self, enable: bool) {
        self.enable_antialiasing = enable;
    }

    /// Checks if antialiasing is enabled
    pub fn antialiasing_enabled(&self) -> bool {
        self.enable_antialiasing
    }

    /// Gets a mutable reference to the base renderer for configuration
    pub fn base_mut(&mut self) -> &mut ImageRenderer {
        &mut self.base
    }

    /// Gets a reference to the base renderer
    pub fn base(&self) -> &ImageRenderer {
        &self.base
    }
}

impl Renderer for AdvancedImageRenderer {
    type Output = RgbImage;

    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output> {
        // For now, just delegate to the base renderer
        // TODO: Add antialiasing and point style selection
        self.base.render(points, camera)
    }

    fn set_viewport(&mut self, width: u32, height: u32) -> Result<()> {
        self.base.set_viewport(width, height)
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.base.viewport_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PointCloud, Color, Camera};
    use glam::Vec3;

    #[test]
    fn test_image_renderer_new() {
        let renderer = ImageRenderer::new(800, 600).unwrap();
        assert_eq!(renderer.viewport_size(), (800, 600));
        assert_eq!(renderer.background_color(), Color::BLACK);
        assert_eq!(renderer.point_size(), 2.0);

        // Test invalid dimensions
        assert!(ImageRenderer::new(0, 600).is_err());
        assert!(ImageRenderer::new(800, 0).is_err());
    }

    #[test]
    fn test_image_renderer_with_background() {
        let renderer = ImageRenderer::with_background(800, 600, Color::WHITE).unwrap();
        assert_eq!(renderer.background_color(), Color::WHITE);
    }

    #[test]
    fn test_set_background_color() {
        let mut renderer = ImageRenderer::new(800, 600).unwrap();
        renderer.set_background_color(Color::BLUE);
        assert_eq!(renderer.background_color(), Color::BLUE);
    }

    #[test]
    fn test_set_point_size() {
        let mut renderer = ImageRenderer::new(800, 600).unwrap();
        assert!(renderer.set_point_size(5.0).is_ok());
        assert_eq!(renderer.point_size(), 5.0);

        // Test invalid size
        assert!(renderer.set_point_size(0.0).is_err());
        assert!(renderer.set_point_size(-1.0).is_err());
    }

    #[test]
    fn test_set_viewport() {
        let mut renderer = ImageRenderer::new(800, 600).unwrap();
        assert!(renderer.set_viewport(1024, 768).is_ok());
        assert_eq!(renderer.viewport_size(), (1024, 768));

        // Test invalid dimensions
        assert!(renderer.set_viewport(0, 768).is_err());
        assert!(renderer.set_viewport(1024, 0).is_err());
    }

    #[test]
    fn test_render_empty_point_cloud() {
        let mut renderer = ImageRenderer::new(100, 100).unwrap();
        let empty_cloud = PointCloud::new();
        let camera = Camera::new();

        let image = renderer.render(&empty_cloud, &camera).unwrap();
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);

        // Should be all background color (black)
        let expected_pixel = Rgb([0, 0, 0]);
        assert_eq!(*image.get_pixel(50, 50), expected_pixel);
    }

    #[test]
    fn test_render_single_point() {
        let mut renderer = ImageRenderer::new(100, 100).unwrap();
        let mut cloud = PointCloud::new();

        // Add a point at the origin
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);

        // Camera looking at origin from positive Z
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let image = renderer.render(&cloud, &camera).unwrap();
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);

        // The red point should be visible somewhere near the center
        // We'll just check that there's at least one red pixel
        let red_pixel = Rgb([255, 0, 0]);
        let mut found_red = false;
        for pixel in image.pixels() {
            if *pixel == red_pixel {
                found_red = true;
                break;
            }
        }
        assert!(found_red, "Red point should be visible in the rendered image");
    }

    #[test]
    fn test_render_multiple_points() {
        let mut renderer = ImageRenderer::new(200, 200).unwrap();
        let mut cloud = PointCloud::new();

        // Add three points in different locations
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);
        cloud.add_point_coords(1.0, 0.0, 0.0, Color::GREEN);
        cloud.add_point_coords(-1.0, 0.0, 0.0, Color::BLUE);

        // Camera looking at origin
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let image = renderer.render(&cloud, &camera).unwrap();

        // Should have pixels of different colors
        let red_pixel = Rgb([255, 0, 0]);
        let green_pixel = Rgb([0, 255, 0]);
        let blue_pixel = Rgb([0, 0, 255]);

        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;

        for pixel in image.pixels() {
            if *pixel == red_pixel { has_red = true; }
            if *pixel == green_pixel { has_green = true; }
            if *pixel == blue_pixel { has_blue = true; }
        }

        assert!(has_red, "Should have red pixels");
        assert!(has_green, "Should have green pixels");
        assert!(has_blue, "Should have blue pixels");
    }

    #[test]
    fn test_render_points_behind_camera() {
        let mut renderer = ImageRenderer::new(100, 100).unwrap();
        let mut cloud = PointCloud::new();

        // Add a point behind the camera
        cloud.add_point_coords(0.0, 0.0, 10.0, Color::RED);

        // Camera at origin looking towards negative Z
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let image = renderer.render(&cloud, &camera).unwrap();

        // Should be all background color (no red pixels)
        let red_pixel = Rgb([255, 0, 0]);
        for pixel in image.pixels() {
            assert_ne!(*pixel, red_pixel, "No red pixels should be visible");
        }
    }

    #[test]
    fn test_draw_point_pixel() {
        let renderer = ImageRenderer::new(10, 10).unwrap();
        let mut image = RgbImage::new(10, 10);

        renderer.draw_point_pixel(&mut image, 5.0, 5.0, Color::RED);

        let red_pixel = Rgb([255, 0, 0]);
        assert_eq!(*image.get_pixel(5, 5), red_pixel);
    }

    #[test]
    fn test_draw_point_square() {
        let renderer = ImageRenderer::new(10, 10).unwrap();
        let mut image = RgbImage::new(10, 10);

        renderer.draw_point_square(&mut image, 5.0, 5.0, 1.0, Color::GREEN);

        let green_pixel = Rgb([0, 255, 0]);
        // Check center and adjacent pixels
        assert_eq!(*image.get_pixel(5, 5), green_pixel);
        assert_eq!(*image.get_pixel(4, 5), green_pixel);
        assert_eq!(*image.get_pixel(6, 5), green_pixel);
        assert_eq!(*image.get_pixel(5, 4), green_pixel);
        assert_eq!(*image.get_pixel(5, 6), green_pixel);
    }

    #[test]
    fn test_draw_point_circle() {
        let renderer = ImageRenderer::new(10, 10).unwrap();
        let mut image = RgbImage::new(10, 10);

        renderer.draw_point(&mut image, 5.0, 5.0, 2.0, Color::BLUE);

        let blue_pixel = Rgb([0, 0, 255]);
        // Check center pixel
        assert_eq!(*image.get_pixel(5, 5), blue_pixel);

        // Check some pixels that should be inside the circle
        assert_eq!(*image.get_pixel(4, 5), blue_pixel);
        assert_eq!(*image.get_pixel(6, 5), blue_pixel);
        assert_eq!(*image.get_pixel(5, 4), blue_pixel);
        assert_eq!(*image.get_pixel(5, 6), blue_pixel);
    }

    #[test]
    fn test_advanced_image_renderer_new() {
        let renderer = AdvancedImageRenderer::new(800, 600).unwrap();
        assert_eq!(renderer.viewport_size(), (800, 600));
        assert_eq!(renderer.point_style(), PointStyle::Circle);
        assert!(!renderer.antialiasing_enabled());
    }

    #[test]
    fn test_advanced_renderer_set_point_style() {
        let mut renderer = AdvancedImageRenderer::new(800, 600).unwrap();
        renderer.set_point_style(PointStyle::Square);
        assert_eq!(renderer.point_style(), PointStyle::Square);
    }

    #[test]
    fn test_advanced_renderer_set_antialiasing() {
        let mut renderer = AdvancedImageRenderer::new(800, 600).unwrap();
        renderer.set_antialiasing(true);
        assert!(renderer.antialiasing_enabled());
    }

    #[test]
    fn test_advanced_renderer_base_access() {
        let mut renderer = AdvancedImageRenderer::new(800, 600).unwrap();

        // Test mutable access
        renderer.base_mut().set_background_color(Color::RED);
        assert_eq!(renderer.base().background_color(), Color::RED);

        // Test immutable access
        assert_eq!(renderer.base().point_size(), 2.0);
    }

    #[test]
    fn test_point_style_enum() {
        // Test enum equality
        assert_eq!(PointStyle::Pixel, PointStyle::Pixel);
        assert_ne!(PointStyle::Pixel, PointStyle::Square);
        assert_ne!(PointStyle::Square, PointStyle::Circle);
    }
}