use std::*;
use process::exit;
use time::Duration;

use crossterm::{
    event,
    execute,
    terminal,
    style,
    cursor
};

mod graphics;
use graphics::*;

// Config
const VIEWPORT_FOV: f32 = 1.7;
const VIEWPORT_DISTANCE: f32 = 0.1;
const TARGET_DURATION_PER_FRAME: Duration = Duration::from_millis(1000 / 60);
const MOUSE_SPEED_MULTIPLIER: f32 = 30.;
const INITIAL_DISTANCE_MULTIPLIER: f32 = 1.5;
const SCROLL_MULTIPLIER: f32 = 0.03;
const PAN_MULTIPLIER: f32 = 0.1;
const HELP_MSG: &str = "\
\x1b[1mAltostratus\x1b[0m: Visualize 3D point files in the terminal!

\x1b[1mUsage\x1b[0m:
    \"altostratus <filepath.txt>\": Interactively view the provided point file.
    \"altostratus --h\", \"altostratus --help\", \"altostratus -h\", \"altostratus -help\", \"altostratus\": Help and info.

\x1b[1mFile Format\x1b[0m:
    Each line should contain three space-separated coordinates: x y z

\x1b[1mControls\x1b[0m:
    Scroll down to zoom out, scroll up to zoom in.
    Click and drag the mouse to rotate around the data.
    Click and drag the mouse while holding [ctrl] to pan.
    Press [/] to enter command mode and load new datasets.
    Press [Ctrl+C] to exit.
";

// Command mode state
struct CommandState {
    active: bool,
    buffer: String,
    error_message: Option<String>,
}

impl CommandState {
    fn new() -> Self {
        CommandState {
            active: false,
            buffer: String::new(),
            error_message: None,
        }
    }

    fn enter_command_mode(&mut self) {
        self.active = true;
        self.buffer.clear();
        self.error_message = None;
    }

    fn exit_command_mode(&mut self) {
        self.active = false;
        self.buffer.clear();
    }

    fn add_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn backspace(&mut self) {
        self.buffer.pop();
    }

    fn execute_command(&mut self, point_cloud: &mut PointCloud) -> bool {
        let command = self.buffer.trim();
        
        if command.starts_with("load ") {
            let path = command.strip_prefix("load ").unwrap().trim();
            match PointCloud::from_file(path) {
                Ok(new_cloud) => {
                    if new_cloud.points.is_empty() {
                        self.error_message = Some("No points found in file".to_string());
                        return false;
                    }
                    
                    // Add new points to existing point cloud
                    point_cloud.points.extend(new_cloud.points);
                    
                    // Regenerate axes based on combined dataset
                    point_cloud.axes = PointCloud::generate_axes_public(&point_cloud.points);
                    
                    self.exit_command_mode();
                    return true; // Signal to reset view parameters
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load: {}", e));
                    return false;
                }
            }
        } else if !command.is_empty() {
            self.error_message = Some("Unknown command".to_string());
            return false;
        }
        
        false
    }

    fn get_display_text(&self) -> String {
        if let Some(ref error) = self.error_message {
            format!("ERROR: {} (press ESC to continue)", error)
        } else {
            format!("Command: {}_", self.buffer)
        }
    }
}

fn graceful_close() -> ! {
    execute!(
        io::stdout(),
        cursor::Show,
        event::DisableMouseCapture,
    ).unwrap();
    terminal::disable_raw_mode().unwrap();
    exit(0)
}

fn error_close(msg: &dyn fmt::Display) -> ! {
    execute!(
        io::stderr(),
        style::Print(msg)
    ).unwrap();
    graceful_close()
}

fn main() {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 { 
        error_close(&"Please supply only one point file path to visualize.") 
    }
    if args.is_empty() { 
        error_close(&"Error parsing arguments.") 
    }
    
    let help_mode = args.len() == 1 || 
        ["-h", "--help"].map(String::from).contains(&args[1]);

    if help_mode {
        execute!(io::stdout(), style::Print(HELP_MSG)).unwrap();
        graceful_close();
    }
        
    terminal::enable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        cursor::Hide,
        event::EnableMouseCapture,
    ).unwrap();

    let file_path = &args[1];
    
    // Load point cloud
    let mut point_cloud = match PointCloud::from_file(file_path) {
        Ok(cloud) => cloud,
        Err(error) => error_close(&error)
    };

    if point_cloud.points.is_empty() {
        error_close(&"No points found in file");
    }

    // Get dimensions
    let mut center = point_cloud.get_center();
    let mut diagonal = point_cloud.get_diagonal().max(1.0); // Ensure we don't get zero diagonal

    // Setup camera
    let mut camera = Camera::new(
        center, 
        0., 0., 0., 
        VIEWPORT_DISTANCE, VIEWPORT_FOV,
    );

    let mut view_yaw: f32 = std::f32::consts::PI / 2.0;
    let mut view_pitch: f32 = 0.0;
    let mut distance_to_data = diagonal * INITIAL_DISTANCE_MULTIPLIER;
    let mut pan_mode = false;

    // Setup events
    let mut mouse_speed: (f32, f32) = (0., 0.);
    let mut last_mouse_position = Point2D::new(0, 0);
    let mut center_point = center;

    // Setup command state
    let mut command_state = CommandState::new();

    // Start main loop
    loop {
        let start = time::Instant::now();
        let mut start_mouse_position = last_mouse_position;

        // Look through the queue while there is an available event
        let mut event_count = 0;
        while event::poll(Duration::from_secs(0)).unwrap() {
            if let Ok(event) = event::read() {
                match event {
                    event::Event::Key(key_event) => {
                        if command_state.active {
                            // Handle command mode input
                            match key_event.code {
                                event::KeyCode::Esc => {
                                    command_state.exit_command_mode();
                                }
                                event::KeyCode::Enter => {
                                    let should_reset_view = command_state.execute_command(&mut point_cloud);
                                    if should_reset_view {
                                        // Reset view parameters for new dataset
                                        center = point_cloud.get_center();
                                        diagonal = point_cloud.get_diagonal().max(1.0);
                                        view_yaw = std::f32::consts::PI / 2.0;
                                        view_pitch = 0.0;
                                        distance_to_data = diagonal * INITIAL_DISTANCE_MULTIPLIER;
                                        center_point = center;
                                    }
                                }
                                event::KeyCode::Backspace => {
                                    command_state.backspace();
                                }
                                event::KeyCode::Char(c) => {
                                    command_state.add_char(c);
                                }
                                _ => {}
                            }
                        } else {
                            // Handle normal mode input
                            let is_ctrl_c = key_event.modifiers == event::KeyModifiers::CONTROL
                                && key_event.code == event::KeyCode::Char('c');

                            if is_ctrl_c { 
                                graceful_close() 
                            } else if key_event.code == event::KeyCode::Char('/') {
                                command_state.enter_command_mode();
                            }
                        }
                    }

                    // Mouse controls (only when not in command mode)
                    event::Event::Mouse(mouse_event) if !command_state.active => {
                        let (x, y) = (mouse_event.column, mouse_event.row);
                        match mouse_event.kind {

                            event::MouseEventKind::Down(_) => {
                                pan_mode = mouse_event.modifiers == event::KeyModifiers::CONTROL;
                                last_mouse_position.x = x as i32;
                                last_mouse_position.y = y as i32;
                                start_mouse_position = last_mouse_position;
                                event_count += 1;
                            }

                            event::MouseEventKind::Drag(_) => {
                                pan_mode = mouse_event.modifiers == event::KeyModifiers::CONTROL;
                                let delta_x = x as f32 - start_mouse_position.x as f32;
                                let delta_y = start_mouse_position.y as f32 - y as f32;
                                mouse_speed.0 = delta_x / camera.screen.width as f32 * MOUSE_SPEED_MULTIPLIER;
                                mouse_speed.1 = delta_y / camera.screen.width as f32 * MOUSE_SPEED_MULTIPLIER;
                                last_mouse_position.x = x as i32;
                                last_mouse_position.y = y as i32;
                                event_count += 1;
                            }

                            event::MouseEventKind::ScrollDown => {
                                distance_to_data += diagonal * SCROLL_MULTIPLIER;
                            }

                            event::MouseEventKind::ScrollUp => {
                                distance_to_data -= diagonal * SCROLL_MULTIPLIER;
                                distance_to_data = distance_to_data.max(0.1);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        // If no event happened, reset the mouse
        if event_count == 0 { 
            mouse_speed = (0., 0.);
            pan_mode = false;
        }

        // Update viewer params
        if pan_mode {
            // Handle horizontal pan
            center_point.x -= mouse_speed.0 * camera.yaw.cos() * diagonal * PAN_MULTIPLIER;
            center_point.z += mouse_speed.0 * camera.yaw.sin() * diagonal * PAN_MULTIPLIER;

            // Handle vertical pan
            center_point.y -= mouse_speed.1 * camera.pitch.cos() * diagonal * PAN_MULTIPLIER;
            center_point.x += mouse_speed.1 * camera.yaw.sin() * camera.pitch.sin() * diagonal * PAN_MULTIPLIER;
            center_point.z += mouse_speed.1 * camera.yaw.cos() * camera.pitch.sin() * diagonal * PAN_MULTIPLIER;
        } else {
            view_yaw -= mouse_speed.0;
            view_pitch -= mouse_speed.1;
        }

        // Update camera position
        camera.coordinates.z = -view_yaw.cos() * view_pitch.cos() * distance_to_data + center_point.z;
        camera.coordinates.x = view_yaw.sin() * view_pitch.cos() * distance_to_data + center_point.x;
        camera.coordinates.y = view_pitch.sin() * distance_to_data + center_point.y;
        camera.yaw = -view_yaw;
        camera.pitch = -view_pitch;

        // Render
        camera.screen.fit_to_terminal();
        camera.screen.clear();

        // Render axes with arrowheads and labels
        for axis in &point_cloud.axes {
            // Draw main axis line
            camera.plot_line(&axis.axis_line.0, &axis.axis_line.1);
            
            // Draw arrowhead lines
            for (start, end) in &axis.arrowhead_lines {
                camera.plot_line(start, end);
            }
        }

        // Render points as vertices
        for point in &point_cloud.points {
            camera.plot_point(point);
        }

        camera.screen.render();
        
        // Add buffer time to hit 60 fps
        if let Some(time) = TARGET_DURATION_PER_FRAME.checked_sub(start.elapsed()) { 
            thread::sleep(time);
        }

        // Status message
        let final_msg = if command_state.active || command_state.error_message.is_some() {
            command_state.get_display_text()
        } else {
            let fps_msg = format!("fps: {:3.0}", 1. / start.elapsed().as_secs_f32());
            let resolution_msg = format!(
                "resolution: {} x {}",
                camera.screen.width,
                camera.screen.height,
            );
            let points_msg = format!("points: {}", point_cloud.points.len());

            let full_msg = format!("{} | {} | {} | Press '/' for commands", points_msg, resolution_msg, fps_msg);
            let short_msg = format!("{} | {} | '/' for commands", points_msg, fps_msg);

            match terminal::size().unwrap().0 as usize {
                width if width > full_msg.len() => full_msg,
                width if width > short_msg.len() => short_msg,
                _ => format!("{} | '/'", points_msg),
            }
        };

        execute!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::Print(final_msg),
        ).unwrap();
    }
}