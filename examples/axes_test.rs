use altostratus::{PointCloud, Camera, ImageRenderer, Renderer, Color, AxesConfig};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing coordinate axes rendering...");

    // Create a simple point cloud with one point at each axis
    let mut cloud = PointCloud::new();
    cloud.add_point_coords(2.0, 0.0, 0.0, Color::new(255, 200, 200)); // Light red on X
    cloud.add_point_coords(0.0, 2.0, 0.0, Color::new(200, 255, 200)); // Light green on Y
    cloud.add_point_coords(0.0, 0.0, 2.0, Color::new(200, 200, 255)); // Light blue on Z
    cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);              // White at origin

    // Position camera to see all axes clearly
    let camera = Camera::look_at(
        Vec3::new(5.0, 4.0, 3.0),  // Camera position
        Vec3::new(0.0, 0.0, 0.0),  // Look at origin
    );

    // Create renderer with axes
    let mut renderer = ImageRenderer::new(600, 600)?;
    renderer.set_background_color(Color::new(30, 30, 50)); // Dark background
    renderer.set_point_size(6.0)?; // Large points for visibility

    // Configure axes with bright colors and features
    let axes_config = AxesConfig::new()
        .with_length(3.0)
        .with_colors(Color::RED, Color::GREEN, Color::BLUE)
        .with_ticks(1.0, 0.2)    // Tick every unit, 0.2 length
        .with_arrow_size(0.3)    // Larger arrows
        .with_features(true, true, true)  // Show ticks, arrows, labels
        .with_resolution(20.0);  // High resolution

    renderer.enable_axes(axes_config);

    println!("Rendering test image...");
    let image = renderer.render(&cloud, &camera)?;
    image.save("axes_test.png")?;

    println!("✓ Saved axes test to: axes_test.png");
    println!("This image should clearly show:");
    println!("  - Red X-axis pointing right with 'X' label");
    println!("  - Green Y-axis pointing up with 'Y' label");
    println!("  - Blue Z-axis pointing toward camera with 'Z' label");
    println!("  - Tick marks at 1, 2, 3 units on each axis");
    println!("  - Arrowheads at the end of each axis");
    println!("  - Colored points positioned on each axis");

    // Test different configurations
    println!("\nTesting different axis configurations...");

    // Minimal axes (no ticks, no labels)
    let minimal_config = AxesConfig::new()
        .with_length(2.5)
        .with_features(false, true, false) // Only arrows
        .with_colors(Color::WHITE, Color::WHITE, Color::WHITE);

    renderer.set_axes_config(Some(minimal_config));
    let image_minimal = renderer.render(&cloud, &camera)?;
    image_minimal.save("axes_minimal.png")?;
    println!("✓ Saved minimal axes to: axes_minimal.png");

    // No axes
    renderer.disable_axes();
    let image_no_axes = renderer.render(&cloud, &camera)?;
    image_no_axes.save("axes_none.png")?;
    println!("✓ Saved no axes to: axes_none.png");

    println!("\nAxes test complete! Compare the three images:");
    println!("  1. axes_test.png - Full axes with all features");
    println!("  2. axes_minimal.png - Minimal white axes");
    println!("  3. axes_none.png - No axes for comparison");

    Ok(())
}