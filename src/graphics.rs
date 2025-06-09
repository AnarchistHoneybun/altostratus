use std::*;
use std::ops;
use crossterm::{execute, terminal, cursor, style};

const DEFAULT_TERMINAL_DIMENSIONS: (u16, u16) = (80, 24);

// Simple 3d point wrapper.
#[derive(Copy, Clone)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Point3D {
        Point3D { x, y, z }
    }
}

// Simple 2d point wrapper.
#[derive(Copy, Clone)]
pub struct Point2D {
    pub x: i32,
    pub y: i32
}

impl Point2D {
    pub fn new(x: i32, y: i32) -> Point2D {
        Point2D { x, y }
    }
}

// Braille pixel struct
#[derive(Clone, Copy)]
pub struct BraillePixel {
    data: [[bool; 2]; 4],
}

impl BraillePixel {
    pub fn new() -> BraillePixel { 
        BraillePixel { data: [[false; 2]; 4] }
    }
    
    pub fn to_char(&self) -> char {
        let mut unicode: u32 = 0;
        if self.data[0][0] { unicode |= 1 << 0 }
        if self.data[1][0] { unicode |= 1 << 1 }
        if self.data[2][0] { unicode |= 1 << 2 }
    
        if self.data[0][1] { unicode |= 1 << 3 }
        if self.data[1][1] { unicode |= 1 << 4 }
        if self.data[2][1] { unicode |= 1 << 5 }
    
        if self.data[3][0] { unicode |= 1 << 6 }
        if self.data[3][1] { unicode |= 1 << 7 }
    
        unicode |= 0x28 << 8;
    
        char::from_u32(unicode).unwrap()
    }
}

impl ops::Index<usize> for BraillePixel {
    type Output = [bool; 2];
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl ops::IndexMut<usize> for BraillePixel {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

// Screen wrapper
pub struct Screen {
    pub width: u16,
    pub height: u16,
    content: Vec<Vec<bool>>,
}

impl Screen {
    pub fn new() -> Screen {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        ).unwrap();

        Screen{
            content: Vec::new(),
            width: 0,
            height: 0
        }
    }

    pub fn fit_to_terminal(&mut self) {
        let (terminal_width, terminal_height) = match terminal::size() {
            Ok(dim) => dim,
            Err(_) => DEFAULT_TERMINAL_DIMENSIONS
        };

        self.resize(
            terminal_width * 2, 
            (terminal_height - 1) * 4
        );
    }

    pub fn write(&mut self, val: bool, point: &Point2D) {
        let x_in_bounds = 0 < point.x && point.x < self.width as i32;
        let y_in_bounds = 0 < point.y && point.y < self.height as i32;
        if x_in_bounds && y_in_bounds {
            self.content[point.y as usize][point.x as usize] = val;
        }
    }

    pub fn clear(&mut self) {
        self.content = vec![vec![false; self.width as usize]; self.height as usize];
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if height > self.height {
            self.content.extend(vec![
                vec![false; width as usize]; 
                (height - self.height) as usize
            ])
        } else {
            self.content.truncate(height as usize);
        }
        self.height = height;

        if width > self.width {
            for row in self.content.iter_mut() {
                row.extend(vec![false; (width - self.width) as usize]);
            }
        } else {
            for row in self.content.iter_mut() {
                row.truncate(width as usize);
            }
        }
        self.width = width;
    }

    pub fn line(&mut self, start: &Point2D, end: &Point2D) {            
        let delta_x = (end.x - start.x).abs();
        let step_x: i32 = if start.x < end.x {1} else {-1};
        let delta_y = -(end.y - start.y).abs();
        let step_y: i32 = if start.y < end.y {1} else {-1};
        let mut err = delta_x + delta_y;

        let mut x = start.x;
        let mut y = start.y;

        self.write(true, &Point2D::new(x, y));

        while !(x == end.x && y == end.y) {
            self.write(true, &Point2D::new(x, y));
            let curr_err = err;

            if 2 * curr_err >= delta_y {
                err += delta_y;
                x += step_x;
            }

            if 2 * curr_err <= delta_x {
                err += delta_x;
                y += step_y;
            }
        }
    }

    pub fn render(&self) {
        execute!(io::stdout(), cursor::MoveTo(0, 0)).unwrap();

        let chunked_rows = self.content.chunks(4);

        for subrows in chunked_rows {
            let real_row_width = self.width.div_ceil(2) as usize;
            let mut real_row = vec![BraillePixel::new(); real_row_width];

            for (subpixel_y, subrow) in subrows.iter().enumerate() {
                let chunked_subrow = subrow.chunks_exact(2);
                let remainder = chunked_subrow.remainder();

                for (real_x, pixel_row) in chunked_subrow.enumerate() {
                    real_row[real_x][subpixel_y][..pixel_row.len()].copy_from_slice(pixel_row);
                }
                
                real_row[real_row_width - 1][subpixel_y][..remainder.len()].copy_from_slice(remainder);
            }

            for pixel in real_row {
                execute!(io::stdout(), style::Print(pixel.to_char())).unwrap();
            }
            execute!(io::stdout(), style::Print("\r\n")).unwrap();
        }
    }
}

pub struct Camera {
    pub coordinates: Point3D,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub viewport_distance: f32,
    pub viewport_fov: f32,
    pub screen: Screen
}

impl Camera {
    pub fn new(
        coordinates: Point3D, 
        yaw: f32, pitch: f32, roll: f32,
        viewport_distance: f32, viewport_fov: f32,
    ) -> Camera {
        Camera { 
            coordinates, 
            yaw, pitch, roll, 
            viewport_distance, viewport_fov, 
            screen: Screen::new()
        }
    }

    fn world_to_camera(&self, point: &Point3D) -> Point3D {
        let (s_yaw, s_pitch, s_roll) = (self.yaw.sin(), self.pitch.sin(), self.roll.sin());
        let (c_yaw, c_pitch, c_roll) = (self.yaw.cos(), self.pitch.cos(), self.roll.cos());

        let delta_x = point.x - self.coordinates.x;
        let delta_y = point.y - self.coordinates.y;
        let delta_z = point.z - self.coordinates.z;

        // Undo yaw
        let unyawed_x = delta_x * c_yaw - delta_z * s_yaw;
        let unyawed_y = delta_y;
        let unyawed_z = delta_x * s_yaw + delta_z * c_yaw;

        // Undo pitch
        let unpitched_x = unyawed_x;
        let unpitched_y = unyawed_y * c_pitch - unyawed_z * s_pitch;
        let unpitched_z = unyawed_y * s_pitch + unyawed_z * c_pitch;

        // Undo roll
        let unrolled_x = unpitched_x * c_roll - unpitched_y * s_roll;
        let unrolled_y = unpitched_x * s_roll + unpitched_y * c_roll;
        let unrolled_z = unpitched_z;

        Point3D::new(unrolled_x, unrolled_y, unrolled_z)
    }

    fn camera_to_screen(&self, point: &Point3D) -> Point2D {
        let viewport_x = point.x * self.viewport_distance / point.z;
        let viewport_y = point.y * self.viewport_distance / point.z;

        let viewport_width = 2. * self.viewport_distance * (self.viewport_fov / 2.).tan();
        let viewport_height = (self.screen.height as f32 / self.screen.width as f32) * viewport_width;

        let screen_x = (viewport_x / viewport_width + 0.5) * self.screen.width as f32;
        let screen_y = (1.0 - (viewport_y / viewport_height + 0.5)) * self.screen.height as f32;

        Point2D::new(screen_x.round() as i32, screen_y.round() as i32)
    }

    pub fn plot_point(&mut self, point: &Point3D) {
        let camera_point = self.world_to_camera(point);
        if camera_point.z >= self.viewport_distance {
            self.screen.write(true, &self.camera_to_screen(&camera_point));
        }
    }

    pub fn plot_line(&mut self, start: &Point3D, end: &Point3D) {
        let camera_start = self.world_to_camera(start);
        let camera_end = self.world_to_camera(end);
        let clip_start = camera_start.z < self.viewport_distance;
        let clip_end = camera_end.z < self.viewport_distance;

        if clip_start && clip_end { return }

        if !clip_start && !clip_end {
            self.screen.line(
                &self.camera_to_screen(&camera_start), 
                &self.camera_to_screen(&camera_end)
            );
            return
        }

        let (clipped, unclipped) = 
            if clip_start { (camera_start, camera_end) } else { (camera_end, camera_start) };

        let distance_behind_viewport = self.viewport_distance - clipped.z;
        let (delta_x, delta_y, delta_z) = (
            unclipped.x - clipped.x,
            unclipped.y - clipped.y,
            unclipped.z - clipped.z
        );
        let lambda = distance_behind_viewport / delta_z;
        let new_clipped = Point3D::new(
            lambda * delta_x + clipped.x, 
            lambda * delta_y + clipped.y, 
            self.viewport_distance
        );

        self.screen.line(
            &self.camera_to_screen(&new_clipped), 
            &self.camera_to_screen(&unclipped)
        )    
    }
}

pub struct AxisDecoration {
    pub axis_line: (Point3D, Point3D),
    pub arrowhead_lines: Vec<(Point3D, Point3D)>,
}

pub struct PointCloud {
    pub points: Vec<Point3D>,
    pub axes: Vec<AxisDecoration>,
}

impl PointCloud {
    pub fn from_file(path: &str) -> Result<PointCloud, Box<dyn error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut points = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            
            let coords: Vec<&str> = line.split_whitespace().collect();
            if coords.len() != 3 {
                return Err(format!("Invalid line format: {}", line).into());
            }
            
            let file_x: f32 = coords[0].parse()?;
let file_y: f32 = coords[1].parse()?;
let file_z: f32 = coords[2].parse()?;

// Remap coordinates: file_z becomes viewer_y (up axis)
points.push(Point3D::new(file_x, file_z, file_y));
        }

        let axes = Self::generate_axes(&points);
        
        Ok(PointCloud { points, axes })
    }

    pub fn generate_axes_public(points: &[Point3D]) -> Vec<AxisDecoration> {
        Self::generate_axes(points)
    }

    fn generate_axes(points: &[Point3D]) -> Vec<AxisDecoration> {
        let max_distance = if points.is_empty() {
            1.0
        } else {
            points.iter()
                .map(|p| (p.x.powi(2) + p.y.powi(2) + p.z.powi(2)).sqrt())
                .fold(0.0, f32::max) * 1.1 // 10% beyond furthest point
        };

        let origin = Point3D::new(0., 0., 0.);
        let x_end = Point3D::new(max_distance, 0., 0.);
        let y_end = Point3D::new(0., max_distance, 0.);
        let z_end = Point3D::new(0., 0., max_distance);

        vec![
            Self::create_axis_decoration(origin, x_end, max_distance),
            Self::create_axis_decoration(origin, y_end, max_distance),
            Self::create_axis_decoration(origin, z_end, max_distance),
        ]
    }

    fn create_axis_decoration(start: Point3D, end: Point3D, scale: f32) -> AxisDecoration {
        let arrowhead_lines = Self::generate_arrowhead(&start, &end, scale);
        
        AxisDecoration {
            axis_line: (start, end),
            arrowhead_lines,
        }
    }

    fn generate_arrowhead(start: &Point3D, end: &Point3D, scale: f32) -> Vec<(Point3D, Point3D)> {
        // Calculate direction vector
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dz = end.z - start.z;
        let length = (dx*dx + dy*dy + dz*dz).sqrt();
        
        if length == 0.0 { return vec![]; }
        
        // Normalized direction
        let dir_x = dx / length;
        let dir_y = dy / length;
        let dir_z = dz / length;
        
        // Arrowhead size
        let arrow_length = scale * 0.05;
        let arrow_angle = 0.5f32; // radians (~30 degrees)
        
        // Find two perpendicular vectors to the axis direction
        let (perp1_x, perp1_y, perp1_z, perp2_x, perp2_y, perp2_z) = if dir_z.abs() < 0.9 {
            // If not too aligned with Z, use Z cross product
            let p1_x = -dir_y;
            let p1_y = dir_x;
            let p1_z = 0.0;
            let p1_len = (p1_x*p1_x + p1_y*p1_y).sqrt();
            let (p1_x, p1_y, p1_z) = if p1_len > 0.0 { (p1_x/p1_len, p1_y/p1_len, p1_z/p1_len) } else { (1.0, 0.0, 0.0) };
            
            // Second perpendicular: dir cross perp1
            let p2_x = dir_y*p1_z - dir_z*p1_y;
            let p2_y = dir_z*p1_x - dir_x*p1_z;
            let p2_z = dir_x*p1_y - dir_y*p1_x;
            
            (p1_x, p1_y, p1_z, p2_x, p2_y, p2_z)
        } else {
            // Use X cross product if aligned with Z
            let p1_x = 0.0;
            let p1_y = -dir_z;
            let p1_z = dir_y;
            let p1_len = (p1_y*p1_y + p1_z*p1_z).sqrt();
            let (p1_x, p1_y, p1_z) = if p1_len > 0.0 { (p1_x/p1_len, p1_y/p1_len, p1_z/p1_len) } else { (0.0, 1.0, 0.0) };
            
            let p2_x = dir_y*p1_z - dir_z*p1_y;
            let p2_y = dir_z*p1_x - dir_x*p1_z;
            let p2_z = dir_x*p1_y - dir_y*p1_x;
            
            (p1_x, p1_y, p1_z, p2_x, p2_y, p2_z)
        };
        
        // Create arrowhead points
        let cos_angle = arrow_angle.cos();
        let sin_angle = arrow_angle.sin();
        
        let arrow1 = Point3D::new(
            end.x - arrow_length * (dir_x * cos_angle + perp1_x * sin_angle),
            end.y - arrow_length * (dir_y * cos_angle + perp1_y * sin_angle),
            end.z - arrow_length * (dir_z * cos_angle + perp1_z * sin_angle),
        );
        
        let arrow2 = Point3D::new(
            end.x - arrow_length * (dir_x * cos_angle + perp2_x * sin_angle),
            end.y - arrow_length * (dir_y * cos_angle + perp2_y * sin_angle),
            end.z - arrow_length * (dir_z * cos_angle + perp2_z * sin_angle),
        );
        
        vec![(*end, arrow1), (*end, arrow2)]
    }


    pub fn get_bounds(&self) -> (Point3D, Point3D) {
        if self.points.is_empty() {
            return (Point3D::new(0., 0., 0.), Point3D::new(0., 0., 0.));
        }

        let mut min_bounds = self.points[0];
        let mut max_bounds = self.points[0];

        for point in &self.points {
            min_bounds.x = f32::min(point.x, min_bounds.x);
            min_bounds.y = f32::min(point.y, min_bounds.y);
            min_bounds.z = f32::min(point.z, min_bounds.z);

            max_bounds.x = f32::max(point.x, max_bounds.x);
            max_bounds.y = f32::max(point.y, max_bounds.y);
            max_bounds.z = f32::max(point.z, max_bounds.z);
        }

        (min_bounds, max_bounds)
    }

    pub fn get_center(&self) -> Point3D {
        let (min_bounds, max_bounds) = self.get_bounds();
        Point3D::new(
            (min_bounds.x + max_bounds.x) / 2.,
            (min_bounds.y + max_bounds.y) / 2.,
            (min_bounds.z + max_bounds.z) / 2.,
        )
    }

    pub fn get_diagonal(&self) -> f32 {
        let (min_bounds, max_bounds) = self.get_bounds();
        (
            (min_bounds.x - max_bounds.x).powi(2) +
            (min_bounds.y - max_bounds.y).powi(2) +
            (min_bounds.z - max_bounds.z).powi(2)
        ).sqrt()
    }
}