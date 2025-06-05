use altostratus::{PointCloud, Camera, ImageRenderer, Renderer, Color, AxesConfig};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Altostratus - Spiral Scene Generator");

    let mut cloud = PointCloud::new();

    // Create a 3D spiral
    let num_points = 1000;
    println!("Generating {} points in a spiral pattern...", num_points);

    for i in 0..num_points {
        let t = i as f32 / num_points as f32;
        let angle = t * 16.0 * std::f32::consts::PI;
        let radius = 3.0 * t; // Expanding radius

        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = t * 6.0; 

        // Color varies along the spiral (HSV-like)
        let hue = t * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        let color = Color::new(r, g, b);

        cloud.add_point_coords(x, y, z, color);
    }

    println!("Created spiral with {} points", cloud.len());

    // Set up camera to view the spiral
    let mut camera = Camera::look_at(
        Vec3::new(8.0, 8.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
    );
    camera.set_fov_degrees(70.0)?;

    println!("Camera positioned at {:?}, looking at origin", camera.position);

    // Create renderer
    let mut renderer = ImageRenderer::new(1024, 768)?;
    renderer.set_background_color(Color::BLACK);
    renderer.set_point_size(2.0)?;

    // Enable coordinate axes to help visualize the 3D spiral
    let axes_config = AxesConfig::new()
        .with_length(7.0)  // Match spiral size
        .with_colors(
            Color::new(255, 80, 80),   // Softer red
            Color::new(80, 255, 80),   // Softer green
            Color::new(80, 80, 255)    // Softer blue
        )
        .with_ticks(2.0, 0.2)  // Tick marks every 2 units
        .with_resolution(20.0); // High resolution for smooth axes

    renderer.enable_axes(axes_config);

    println!("Rendering 1024x768 image with coordinate axes...");

    // Render
    let image = renderer.render(&cloud, &camera)?;
    image.save("spiral.png")?;

    println!("✓ Saved spiral scene to: spiral.png");

    // Create additional views with different camera angles
    println!("\nGenerating additional views...");

    // Side view
    camera.set_position(Vec3::new(15.0, 0.0, 0.0));
    let image_side = renderer.render(&cloud, &camera)?;
    image_side.save("spiral_side.png")?;
    println!("✓ Saved side view to: spiral_side.png");

    // Top view
    camera.set_position(Vec3::new(0.1, 15.0, 0.0));
    let image_top = renderer.render(&cloud, &camera)?;
    image_top.save("spiral_top.png")?;
    println!("✓ Saved top view to: spiral_top.png");

    // Close-up view with larger points
    camera.set_position(Vec3::new(4.0, 4.0, 4.0));
    renderer.set_point_size(6.0)?;
    renderer.set_background_color(Color::new(10, 10, 30)); // Dark blue
    let image_closeup = renderer.render(&cloud, &camera)?;
    image_closeup.save("spiral_closeup.png")?;
    println!("✓ Saved close-up view to: spiral_closeup.png");

    println!("\nSpiral scene generation complete!");
    println!("Generated files:");
    println!("  - spiral.png (main view)");
    println!("  - spiral_side.png (side view)");
    println!("  - spiral_top.png (top view)");
    println!("  - spiral_closeup.png (close-up with large points)");
    println!("\nAll images include coordinate axes:");
    println!("  - Red axis = X direction");
    println!("  - Green axis = Y direction");
    println!("  - Blue axis = Z direction");
    println!("  - The spiral rotates around Z and expands in the XY plane");

    Ok(())
}

/// Simple HSV to RGB conversion
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;

    (r, g, b)
}
