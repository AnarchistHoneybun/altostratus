use crate::{Renderer, PointCloud, Camera, Color, Result, AltostratusError, Projector, DepthBuffer, AxesConfig, Axes};

/// ASCII character sets for different density styles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterSet {
    /// Simple density progression: " .:-=+*#%@"
    Standard,
    /// Shaded blocks: " ░▒▓█"
    Blocks,
    /// Dots and circles: " .·•○●"
    Dots,
    /// Custom character set
    Custom(Vec<char>),
}

impl CharacterSet {
    /// Gets the characters for this set, from lightest to darkest
    pub fn chars(&self) -> Vec<char> {
        match self {
            CharacterSet::Standard => " .:-=+*#%@".chars().collect(),
            CharacterSet::Blocks => " ░▒▓█".chars().collect(),
            CharacterSet::Dots => " .·•○●".chars().collect(),
            CharacterSet::Custom(chars) => chars.clone(),
        }
    }

    /// Gets the number of density levels in this character set
    pub fn levels(&self) -> usize {
        self.chars().len()
    }
}

impl Default for CharacterSet {
    fn default() -> Self {
        CharacterSet::Standard
    }
}

/// ASCII renderer that outputs text-based visualizations
#[derive(Debug)]
pub struct AsciiRenderer {
    /// Terminal width in characters
    width: u32,
    /// Terminal height in characters
    height: u32,
    /// Character set to use for rendering
    character_set: CharacterSet,
    /// Background character (usually space)
    background_char: char,
    /// Whether to use color codes in output
    use_color: bool,
    /// 3D to 2D projector
    projector: Projector,
    /// Depth buffer for proper point ordering
    depth_buffer: DepthBuffer,
    /// Axes configuration (None = no axes)
    axes_config: Option<AxesConfig>,
}

impl AsciiRenderer {
    /// Creates a new ASCII renderer with the given dimensions
    ///
    /// # Arguments
    /// * `width` - Terminal width in characters
    /// * `height` - Terminal height in characters
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let projector = Projector::new(width, height)?;
        let depth_buffer = DepthBuffer::new(width, height)?;

        Ok(Self {
            width,
            height,
            character_set: CharacterSet::default(),
            background_char: ' ',
            use_color: false,
            projector,
            depth_buffer,
            axes_config: None,
        })
    }

    /// Creates a new ASCII renderer with color support
    ///
    /// # Arguments
    /// * `width` - Terminal width in characters
    /// * `height` - Terminal height in characters
    pub fn with_color(width: u32, height: u32) -> Result<Self> {
        let mut renderer = Self::new(width, height)?;
        renderer.use_color = true;
        Ok(renderer)
    }

    /// Sets the character set for rendering
    ///
    /// # Arguments
    /// * `character_set` - Character set to use
    pub fn set_character_set(&mut self, character_set: CharacterSet) {
        self.character_set = character_set;
    }

    /// Gets the current character set
    pub fn character_set(&self) -> &CharacterSet {
        &self.character_set
    }

    /// Sets the background character
    ///
    /// # Arguments
    /// * `ch` - Background character (usually ' ' or '.')
    pub fn set_background_char(&mut self, ch: char) {
        self.background_char = ch;
    }

    /// Gets the current background character
    pub fn background_char(&self) -> char {
        self.background_char
    }

    /// Enables or disables color output
    ///
    /// # Arguments
    /// * `enable` - Whether to use ANSI color codes
    pub fn set_color_enabled(&mut self, enable: bool) {
        self.use_color = enable;
    }

    /// Checks if color output is enabled
    pub fn color_enabled(&self) -> bool {
        self.use_color
    }

    /// Enables coordinate axes with the given configuration
    ///
    /// # Arguments
    /// * `config` - Axes configuration
    pub fn enable_axes(&mut self, config: AxesConfig) {
        self.axes_config = Some(config);
    }

    /// Enables coordinate axes with default configuration
    pub fn enable_default_axes(&mut self) {
        self.axes_config = Some(AxesConfig::default());
    }

    /// Disables coordinate axes
    pub fn disable_axes(&mut self) {
        self.axes_config = None;
    }

    /// Gets the current axes configuration
    pub fn axes_config(&self) -> Option<&AxesConfig> {
        self.axes_config.as_ref()
    }

    /// Sets a custom axes configuration
    ///
    /// # Arguments
    /// * `config` - New axes configuration (None to disable)
    pub fn set_axes_config(&mut self, config: Option<AxesConfig>) {
        self.axes_config = config;
    }

    /// Maps a depth value to a character from the current character set
    ///
    /// # Arguments
    /// * `depth` - Depth value (0.0 = near, 1.0 = far)
    fn depth_to_char(&self, depth: f32) -> char {
        let chars = self.character_set.chars();
        if chars.is_empty() {
            return ' ';
        }

        // Map depth to character index (inverted so closer = denser character)
        // Adjust the depth range to make most visible points use visible characters
        // Instead of using full 0.0-1.0 range, use a more practical range like 0.0-0.95
        let practical_far = 0.95; // Points beyond this depth are considered "far"
        let clamped_depth = depth.clamp(0.0, practical_far);
        let normalized_depth = clamped_depth / practical_far;
        let inverted_depth = 1.0 - normalized_depth; // Closer objects are "denser"

        // Map to character index, but skip the first character (space) for visible points
        // Use indices 1 to chars.len()-1 for visible characters
        if chars.len() <= 1 {
            return chars[0];
        }

        let visible_chars = &chars[1..]; // Skip space character for visible points
        let index = (inverted_depth * (visible_chars.len() - 1) as f32) as usize;
        let index = index.min(visible_chars.len() - 1);

        visible_chars[index]
    }

    /// Debug version of depth_to_char that prints mapping info
    #[allow(dead_code)]
    fn depth_to_char_debug(&self, depth: f32) -> char {
        let chars = self.character_set.chars();
        if chars.is_empty() {
            println!("  depth_to_char: empty character set, returning space");
            return ' ';
        }

        // Use the same logic as the main depth_to_char function
        let practical_far = 0.95;
        let clamped_depth = depth.clamp(0.0, practical_far);
        let normalized_depth = clamped_depth / practical_far;
        let inverted_depth = 1.0 - normalized_depth;

        if chars.len() <= 1 {
            let ch = chars[0];
            println!("  depth_to_char: only one character available: '{}'", ch);
            return ch;
        }

        let visible_chars = &chars[1..];
        let index = (inverted_depth * (visible_chars.len() - 1) as f32) as usize;
        let index = index.min(visible_chars.len() - 1);
        let ch = visible_chars[index];

        println!("  depth_to_char: depth={:.3} -> clamped={:.3} -> normalized={:.3} -> inverted={:.3} -> index={} -> char='{}'",
                 depth, clamped_depth, normalized_depth, inverted_depth, index, ch);

        ch
    }

    /// Converts an RGB color to ANSI color code
    ///
    /// # Arguments
    /// * `color` - RGB color
    fn color_to_ansi(&self, color: Color) -> String {
        if !self.use_color {
            return String::new();
        }

        // Use 256-color ANSI codes for better color representation
        // Convert RGB to closest ANSI 256-color code
        let r = (color.r as f32 / 255.0 * 5.0) as u8;
        let g = (color.g as f32 / 255.0 * 5.0) as u8;
        let b = (color.b as f32 / 255.0 * 5.0) as u8;
        let ansi_code = 16 + 36 * r + 6 * g + b;

        format!("\x1b[38;5;{}m", ansi_code)
    }

    /// Resets ANSI color codes
    fn reset_color(&self) -> &'static str {
        if self.use_color {
            "\x1b[0m"
        } else {
            ""
        }
    }
}

impl Renderer for AsciiRenderer {
    type Output = String;

    /// Renders a point cloud to an ASCII string
    ///
    /// # Arguments
    /// * `points` - Point cloud to render
    /// * `camera` - Camera defining the view
    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output> {
        // Combine user points with axes if enabled
        let render_cloud = if let Some(ref axes_config) = self.axes_config {
            let axes = Axes::new(axes_config.clone());
            let axes_points = axes.generate_points();

            let mut combined_cloud = points.clone();
            for point in axes_points.iter() {
                combined_cloud.add_point(*point);
            }
            combined_cloud
        } else {
            points.clone()
        };

        // Create character and color buffers
        let mut char_buffer: Vec<Vec<char>> = vec![vec![self.background_char; self.width as usize]; self.height as usize];
        let mut color_buffer: Vec<Vec<Color>> = vec![vec![Color::WHITE; self.width as usize]; self.height as usize];

        if render_cloud.is_empty() {
            // Return empty buffer
            return Ok(self.buffer_to_string(&char_buffer, &color_buffer));
        }

        // Update camera's aspect ratio to match our dimensions
        // Terminal characters are typically ~2:1 height:width ratio
        // So visual aspect ratio = width / (height * 0.5)
        let mut render_camera = camera.clone();
        let visual_aspect_ratio = (self.width as f32) / (self.height as f32 * 0.5);
        render_camera.set_aspect_ratio(visual_aspect_ratio)?;

        // Project all points to screen coordinates
        let projected_points = self.projector.project_point_cloud(&render_cloud, &render_camera);

        if projected_points.is_empty() {
            // No visible points - return background
            return Ok(self.buffer_to_string(&char_buffer, &color_buffer));
        }

        // Clear depth buffer
        self.depth_buffer.clear();

        // Sort points by depth (back to front for proper rendering)
        let mut sorted_points = projected_points;
        sorted_points.sort_by(|a, b| b.1.depth.partial_cmp(&a.1.depth).unwrap_or(std::cmp::Ordering::Equal));

        // Render points to character buffer
        for (point3d, screen_point) in sorted_points {
            // Check bounds as floats BEFORE casting to avoid u32 overflow with negative numbers
            if screen_point.x < 0.0 || screen_point.y < 0.0 ||
                screen_point.x >= self.width as f32 || screen_point.y >= self.height as f32 {
                continue;
            }

            let x = screen_point.x.round() as u32;
            let y = screen_point.y.round() as u32;

            // Double-check bounds (should be redundant now, but safe)
            if x >= self.width || y >= self.height {
                continue;
            }

            // Depth test
            if self.depth_buffer.test_and_update(x, y, screen_point.depth) {
                let ch = self.depth_to_char(screen_point.depth);
                char_buffer[y as usize][x as usize] = ch;
                color_buffer[y as usize][x as usize] = point3d.color;
            }
        }

        Ok(self.buffer_to_string(&char_buffer, &color_buffer))
    }

    /// Sets the viewport size (terminal dimensions)
    ///
    /// # Arguments
    /// * `width` - New terminal width in characters
    /// * `height` - New terminal height in characters
    fn set_viewport(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(AltostratusError::InvalidParameter(
                "Terminal dimensions must be positive".to_string()
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

impl AsciiRenderer {
    /// Debug version of render that prints detailed information
    #[allow(dead_code)]
    pub fn render_debug(&mut self, points: &PointCloud, camera: &Camera) -> crate::Result<String> {
        println!("=== ASCII Renderer Debug ===");
        println!("Viewport: {}x{}", self.width, self.height);
        println!("Camera: pos={:?}, target={:?}", camera.position, camera.target);

        // Combine user points with axes if enabled
        let render_cloud = if let Some(ref axes_config) = self.axes_config {
            let axes = Axes::new(axes_config.clone());
            let axes_points = axes.generate_points();

            let mut combined_cloud = points.clone();
            for point in axes_points.iter() {
                combined_cloud.add_point(*point);
            }
            println!("Combined cloud: {} user points + {} axis points = {} total",
                     points.len(), axes_points.len(), combined_cloud.len());
            combined_cloud
        } else {
            println!("Point cloud: {} points (no axes)", points.len());
            points.clone()
        };

        // Create character and color buffers
        let mut char_buffer: Vec<Vec<char>> = vec![vec![self.background_char; self.width as usize]; self.height as usize];
        let mut color_buffer: Vec<Vec<Color>> = vec![vec![Color::WHITE; self.width as usize]; self.height as usize];

        if render_cloud.is_empty() {
            println!("Empty cloud, returning background");
            return Ok(self.buffer_to_string(&char_buffer, &color_buffer));
        }

        // Update camera's aspect ratio to match our dimensions
        // Terminal characters are typically ~2:1 height:width ratio
        let mut render_camera = camera.clone();
        let visual_aspect_ratio = (self.width as f32) / (self.height as f32 * 0.5);
        render_camera.set_aspect_ratio(visual_aspect_ratio)?;
        println!("Updated camera aspect ratio to: {:.2} (accounting for terminal character proportions)", visual_aspect_ratio);

        // Project all points to screen coordinates
        let projected_points = self.projector.project_point_cloud(&render_cloud, &render_camera);
        println!("Projected {} out of {} points", projected_points.len(), render_cloud.len());

        for (i, (point3d, screen_point)) in projected_points.iter().enumerate().take(5) {
            println!("  Point {}: 3D={:?} -> 2D=({:.2}, {:.2}, depth={:.3})",
                     i, point3d.position, screen_point.x, screen_point.y, screen_point.depth);
        }
        if projected_points.len() > 5 {
            println!("  ... and {} more points", projected_points.len() - 5);
        }

        if projected_points.is_empty() {
            println!("No visible points, returning background");
            return Ok(self.buffer_to_string(&char_buffer, &color_buffer));
        }

        // Clear depth buffer
        self.depth_buffer.clear();

        // Sort points by depth (back to front for proper rendering)
        let mut sorted_points = projected_points;
        sorted_points.sort_by(|a, b| b.1.depth.partial_cmp(&a.1.depth).unwrap_or(std::cmp::Ordering::Equal));

        println!("Rendering {} points to character buffer", sorted_points.len());

        let mut rendered_count = 0;
        let mut out_of_bounds_count = 0;
        let mut depth_failed_count = 0;

        // Render points to character buffer
        for (point3d, screen_point) in sorted_points {
            // Check bounds as floats BEFORE casting to avoid u32 overflow with negative numbers
            if screen_point.x < 0.0 || screen_point.y < 0.0 ||
                screen_point.x >= self.width as f32 || screen_point.y >= self.height as f32 {
                out_of_bounds_count += 1;
                continue;
            }

            let x = screen_point.x.round() as u32;
            let y = screen_point.y.round() as u32;

            // Double-check bounds (should be redundant now, but safe)
            if x >= self.width || y >= self.height {
                out_of_bounds_count += 1;
                continue;
            }

            // Depth test
            if self.depth_buffer.test_and_update(x, y, screen_point.depth) {
                let ch = self.depth_to_char(screen_point.depth);
                char_buffer[y as usize][x as usize] = ch;
                color_buffer[y as usize][x as usize] = point3d.color;
                rendered_count += 1;

                if rendered_count <= 3 {
                    println!("  Rendered point at ({}, {}) depth={:.3} char='{}'",
                             x, y, screen_point.depth, ch);
                }
            } else {
                depth_failed_count += 1;
            }
        }

        println!("Rendering summary:");
        println!("  Successfully rendered: {}", rendered_count);
        println!("  Out of bounds: {}", out_of_bounds_count);
        println!("  Failed depth test: {}", depth_failed_count);

        Ok(self.buffer_to_string(&char_buffer, &color_buffer))
    }

    /// Converts character and color buffers to a formatted string
    ///
    /// # Arguments
    /// * `char_buffer` - 2D array of characters
    /// * `color_buffer` - 2D array of colors
    fn buffer_to_string(&self, char_buffer: &[Vec<char>], color_buffer: &[Vec<Color>]) -> String {
        let mut result = String::new();

        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                let ch = char_buffer[y][x];
                let color = color_buffer[y][x];

                if self.use_color && ch != self.background_char {
                    result.push_str(&self.color_to_ansi(color));
                    result.push(ch);
                    result.push_str(self.reset_color());
                } else {
                    result.push(ch);
                }
            }
            result.push('\n');
        }

        // Remove the final newline
        if result.ends_with('\n') {
            result.pop();
        }

        result
    }
}

/// Extended ASCII renderer with additional formatting options
#[derive(Debug)]
pub struct AdvancedAsciiRenderer {
    base: AsciiRenderer,
    /// Whether to add a border around the output
    show_border: bool,
    /// Border character to use
    border_char: char,
    /// Whether to show coordinate information
    show_info: bool,
}

impl AdvancedAsciiRenderer {
    /// Creates a new advanced ASCII renderer
    ///
    /// # Arguments
    /// * `width` - Terminal width in characters
    /// * `height` - Terminal height in characters
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            base: AsciiRenderer::new(width, height)?,
            show_border: false,
            border_char: '#',
            show_info: false,
        })
    }

    /// Enables or disables border around the output
    ///
    /// # Arguments
    /// * `show` - Whether to show border
    /// * `border_char` - Character to use for border
    pub fn set_border(&mut self, show: bool, border_char: char) {
        self.show_border = show;
        self.border_char = border_char;
    }

    /// Enables or disables coordinate information display
    ///
    /// # Arguments
    /// * `show` - Whether to show coordinate info
    pub fn set_info(&mut self, show: bool) {
        self.show_info = show;
    }

    /// Gets a mutable reference to the base renderer
    pub fn base_mut(&mut self) -> &mut AsciiRenderer {
        &mut self.base
    }

    /// Gets a reference to the base renderer
    pub fn base(&self) -> &AsciiRenderer {
        &self.base
    }

    /// Adds border to rendered output
    fn add_border(&self, content: &str) -> String {
        if !self.show_border {
            return content.to_string();
        }

        let lines: Vec<&str> = content.lines().collect();
        let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);

        let mut result = String::new();

        // Top border
        result.push_str(&self.border_char.to_string().repeat(max_width + 2));
        result.push('\n');

        // Content with side borders
        for line in lines {
            result.push(self.border_char);
            result.push_str(line);
            // Pad to max width
            result.push_str(&" ".repeat(max_width.saturating_sub(line.len())));
            result.push(self.border_char);
            result.push('\n');
        }

        // Bottom border
        result.push_str(&self.border_char.to_string().repeat(max_width + 2));

        result
    }

    /// Adds coordinate information header
    fn add_info(&self, content: &str, camera: &Camera) -> String {
        if !self.show_info {
            return content.to_string();
        }

        let mut result = String::new();

        result.push_str(&format!("Camera: pos={:.1?}, target={:.1?}\n",
                                 camera.position, camera.target));
        result.push_str(&format!("FOV: {:.1}°, Distance: {:.1}\n",
                                 camera.fov_degrees(), camera.distance_to_target()));
        result.push_str(&format!("Viewport: {}x{}\n", self.base.width, self.base.height));
        result.push_str("=".repeat(40).as_str());
        result.push('\n');
        result.push_str(content);

        result
    }
}

impl Renderer for AdvancedAsciiRenderer {
    type Output = String;

    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output> {
        let base_output = self.base.render(points, camera)?;
        let with_border = self.add_border(&base_output);
        let with_info = self.add_info(&with_border, camera);
        Ok(with_info)
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
    fn test_character_set_standard() {
        let charset = CharacterSet::Standard;
        let chars = charset.chars();
        assert_eq!(chars, " .:-=+*#%@".chars().collect::<Vec<_>>());
        assert_eq!(charset.levels(), 10);
    }

    #[test]
    fn test_character_set_blocks() {
        let charset = CharacterSet::Blocks;
        let chars = charset.chars();
        assert_eq!(chars, " ░▒▓█".chars().collect::<Vec<_>>());
        assert_eq!(charset.levels(), 5);
    }

    #[test]
    fn test_character_set_custom() {
        let custom_chars = vec!['A', 'B', 'C'];
        let charset = CharacterSet::Custom(custom_chars.clone());
        assert_eq!(charset.chars(), custom_chars);
        assert_eq!(charset.levels(), 3);
    }

    #[test]
    fn test_ascii_renderer_new() {
        let renderer = AsciiRenderer::new(80, 24).unwrap();
        assert_eq!(renderer.viewport_size(), (80, 24));
        assert_eq!(renderer.background_char(), ' ');
        assert!(!renderer.color_enabled());
        assert!(renderer.axes_config().is_none());

        // Test invalid dimensions
        assert!(AsciiRenderer::new(0, 24).is_err());
        assert!(AsciiRenderer::new(80, 0).is_err());
    }

    #[test]
    fn test_ascii_renderer_with_color() {
        let renderer = AsciiRenderer::with_color(80, 24).unwrap();
        assert!(renderer.color_enabled());
    }

    #[test]
    fn test_set_character_set() {
        let mut renderer = AsciiRenderer::new(80, 24).unwrap();
        renderer.set_character_set(CharacterSet::Blocks);

        match renderer.character_set() {
            CharacterSet::Blocks => (),
            _ => panic!("Character set not set correctly"),
        }
    }

    #[test]
    fn test_set_background_char() {
        let mut renderer = AsciiRenderer::new(80, 24).unwrap();
        renderer.set_background_char('.');
        assert_eq!(renderer.background_char(), '.');
    }

    #[test]
    fn test_set_color_enabled() {
        let mut renderer = AsciiRenderer::new(80, 24).unwrap();
        assert!(!renderer.color_enabled());

        renderer.set_color_enabled(true);
        assert!(renderer.color_enabled());

        renderer.set_color_enabled(false);
        assert!(!renderer.color_enabled());
    }

    #[test]
    fn test_axes_configuration() {
        let mut renderer = AsciiRenderer::new(80, 24).unwrap();

        // Initially no axes
        assert!(renderer.axes_config().is_none());

        // Enable default axes
        renderer.enable_default_axes();
        assert!(renderer.axes_config().is_some());

        // Enable custom axes
        let custom_config = AxesConfig::new().with_length(10.0);
        renderer.enable_axes(custom_config.clone());
        assert!(renderer.axes_config().is_some());
        assert_eq!(renderer.axes_config().unwrap().length, 10.0);

        // Disable axes
        renderer.disable_axes();
        assert!(renderer.axes_config().is_none());
    }

    #[test]
    fn test_depth_to_char() {
        let renderer = AsciiRenderer::new(10, 10).unwrap();

        // Test depth mapping (closer = denser character, but never space for visible points)
        let char_near = renderer.depth_to_char(0.0); // Very close
        let char_mid = renderer.depth_to_char(0.5);  // Middle distance
        let char_far = renderer.depth_to_char(0.95); // Far (at practical limit)
        let char_very_far = renderer.depth_to_char(1.0); // Beyond practical limit

        let chars = renderer.character_set().chars();
        let visible_chars = &chars[1..]; // Skip space character

        // Near should be densest visible character
        assert_eq!(char_near, visible_chars[visible_chars.len() - 1]);

        // Far should be lightest visible character 
        assert_eq!(char_far, visible_chars[0]);
        assert_eq!(char_very_far, visible_chars[0]); // Clamped to practical_far

        // All visible points should get non-space characters
        assert_ne!(char_near, ' ');
        assert_ne!(char_mid, ' ');
        assert_ne!(char_far, ' ');
        assert_ne!(char_very_far, ' ');
    }

    #[test]
    fn test_color_to_ansi() {
        let renderer = AsciiRenderer::with_color(10, 10).unwrap();

        let red_ansi = renderer.color_to_ansi(Color::RED);
        assert!(red_ansi.contains("\x1b[38;5;"));

        let renderer_no_color = AsciiRenderer::new(10, 10).unwrap();
        let no_ansi = renderer_no_color.color_to_ansi(Color::RED);
        assert!(no_ansi.is_empty());
    }

    #[test]
    fn test_render_empty_point_cloud() {
        let mut renderer = AsciiRenderer::new(10, 10).unwrap();
        let empty_cloud = PointCloud::new();
        let camera = Camera::new();

        let output = renderer.render(&empty_cloud, &camera).unwrap();

        // Should be all background characters
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 10);
        for line in lines {
            assert_eq!(line, "          "); // 10 spaces
        }
    }

    #[test]
    fn test_render_single_point() {
        let mut renderer = AsciiRenderer::new(20, 20).unwrap();
        let mut cloud = PointCloud::new();

        // Add a point at the origin
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);

        // Camera looking at origin from positive Z
        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        // Use debug version to see what's happening
        let output = renderer.render_debug(&cloud, &camera).unwrap();

        // Debug: Print the actual output to see what we got
        println!("Final rendered output:");
        println!("┌{}┐", "─".repeat(20));
        for (i, line) in output.lines().enumerate() {
            println!("│{}│ {}", line, i);
        }
        println!("└{}┘", "─".repeat(20));

        // Count different types of characters
        let total_chars = output.chars().count();
        let spaces = output.chars().filter(|&c| c == ' ').count();
        let newlines = output.chars().filter(|&c| c == '\n').count();
        let other_chars = total_chars - spaces - newlines;

        println!("Character breakdown:");
        println!("  Total chars: {}", total_chars);
        println!("  Spaces: {}", spaces);
        println!("  Newlines: {}", newlines);
        println!("  Other chars: {}", other_chars);

        // Should contain at least one non-space character
        assert!(output.chars().any(|c| c != ' ' && c != '\n'),
                "Expected at least one visible character, but got only spaces and newlines");

        // Should be roughly 20 lines
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 20);
    }

    #[test]
    fn test_render_with_axes() {
        let mut renderer = AsciiRenderer::new(30, 20).unwrap();
        renderer.enable_default_axes();

        let mut cloud = PointCloud::new();
        cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);

        let camera = Camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);

        let output = renderer.render(&cloud, &camera).unwrap();

        // Should have more visible characters due to axes
        let visible_chars = output.chars().filter(|&c| c != ' ' && c != '\n').count();
        assert!(visible_chars > 10, "Should have visible axis characters");
    }

    #[test]
    fn test_set_viewport() {
        let mut renderer = AsciiRenderer::new(80, 24).unwrap();
        assert!(renderer.set_viewport(100, 30).is_ok());
        assert_eq!(renderer.viewport_size(), (100, 30));

        // Test invalid dimensions
        assert!(renderer.set_viewport(0, 30).is_err());
        assert!(renderer.set_viewport(100, 0).is_err());
    }

    #[test]
    fn test_advanced_ascii_renderer_new() {
        let renderer = AdvancedAsciiRenderer::new(80, 24).unwrap();
        assert_eq!(renderer.viewport_size(), (80, 24));
    }

    #[test]
    fn test_advanced_renderer_border() {
        let mut renderer = AdvancedAsciiRenderer::new(10, 5).unwrap();
        renderer.set_border(true, '#');

        let cloud = PointCloud::new();
        let camera = Camera::new();

        let output = renderer.render(&cloud, &camera).unwrap();

        // Should contain border characters
        assert!(output.contains('#'));

        // Check that border is properly formed
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() > 5); // Should have more lines due to border
    }

    #[test]
    fn test_advanced_renderer_info() {
        let mut renderer = AdvancedAsciiRenderer::new(10, 5).unwrap();
        renderer.set_info(true);

        let cloud = PointCloud::new();
        let camera = Camera::new();

        let output = renderer.render(&cloud, &camera).unwrap();

        // Should contain camera information
        assert!(output.contains("Camera:"));
        assert!(output.contains("FOV:"));
        assert!(output.contains("Viewport:"));
    }

    #[test]
    fn test_character_set_default() {
        let charset = CharacterSet::default();
        match charset {
            CharacterSet::Standard => (),
            _ => panic!("Default should be Standard"),
        }
    }
}