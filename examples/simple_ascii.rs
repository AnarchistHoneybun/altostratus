use altostratus::{PointCloud, Camera, AsciiRenderer, Renderer, Color, AxesConfig};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple ASCII Rendering Test");
    println!("===========================\n");

    // Create a simple 3D scene
    let mut cloud = PointCloud::new();

    // Add a few key points
    cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);    // Origin
    cloud.add_point_coords(2.0, 0.0, 0.0, Color::RED);      // X axis
    cloud.add_point_coords(0.0, 2.0, 0.0, Color::GREEN);    // Y axis
    cloud.add_point_coords(0.0, 0.0, 2.0, Color::BLUE);     // Z axis
    cloud.add_point_coords(1.0, 1.0, 1.0, Color::RED);   // Corner

    // Position camera to see the scene
    let camera = Camera::look_at(
        Vec3::new(4.0, 3.0, 5.0),  // Camera position
        Vec3::new(0.0, 0.0, 0.0),  // Look at origin
    );

    // Create ASCII renderer with coordinate axes
    let mut renderer = AsciiRenderer::new(50, 25)?;

    // Configure axes for ASCII rendering
    let axes_config = AxesConfig::new()
        .with_length(3.0)
        .with_ticks(1.0, 0.1)
        .with_resolution(15.0);  // Good resolution for ASCII

    renderer.enable_axes(axes_config);

    println!("Rendering 3D scene as ASCII (50x25 characters):");
    println!("- White point at origin");
    println!("- Red, Green, Blue points on axes");
    println!("- Yellow point at (1,1,1)");
    println!("- Coordinate axes with tick marks\n");

    // Render the scene
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    println!("\nKey:");
    println!("  Characters closer to camera: @%#*+=-:. ");
    println!("  Characters farther from camera: .:-=+*#%@");
    println!("  Red axis = X, Green axis = Y, Blue axis = Z");

    Ok(())
}