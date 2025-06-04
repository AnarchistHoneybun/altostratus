use altostratus::{PointCloud, Camera, ImageRenderer, Renderer, Color, AxesConfig};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Altostratus 3D Plotting Tool - Basic Example");

    // Create a point cloud with some sample data
    let mut cloud = PointCloud::new();

    // Add a few colorful points in 3D space
    cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);        // Origin - red
    cloud.add_point_coords(1.0, 0.0, 0.0, Color::GREEN);      // X-axis - green  
    cloud.add_point_coords(0.0, 1.0, 0.0, Color::BLUE);       // Y-axis - blue
    cloud.add_point_coords(0.0, 0.0, 1.0, Color::WHITE);      // Z-axis - white
    cloud.add_point_coords(-1.0, -1.0, -1.0, Color::GRAY);    // Negative corner

    // Add some points in a circle pattern
    let num_circle_points = 12;
    for i in 0..num_circle_points {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / num_circle_points as f32;
        let x = 2.0 * angle.cos();
        let y = 2.0 * angle.sin();
        let z = 0.5;

        // Color varies with angle
        let red = ((angle.sin() + 1.0) * 127.5) as u8;
        let green = ((angle.cos() + 1.0) * 127.5) as u8;
        let blue = 128;
        let color = Color::new(red, green, blue);

        cloud.add_point_coords(x, y, z, color);
    }

    println!("Created point cloud with {} points", cloud.len());

    // Set up a camera
    let mut camera = Camera::look_at(
        Vec3::new(4.0, 3.0, 5.0),  // Camera position
        Vec3::new(0.0, 0.0, 0.0),  // Look at origin
    );
    camera.set_fov_degrees(45.0)?;

    // Auto-frame the camera to show all points
    if let Some((min, max)) = cloud.bounding_box() {
        camera.frame_bounding_box(min, max)?;
        println!("Auto-framed camera to show bounding box: {:?} to {:?}", min, max);
    }

    // Create an image renderer
    let mut renderer = ImageRenderer::new(800, 600)?;
    renderer.set_background_color(Color::new(20, 20, 40)); // Dark blue background
    renderer.set_point_size(4.0)?; // Larger points for visibility

    // Enable coordinate axes with custom configuration
    let axes_config = AxesConfig::new()
        .with_length(3.0)  // Shorter axes for better framing
        .with_colors(Color::new(255, 100, 100), Color::new(100, 255, 100), Color::new(100, 100, 255)) // Softer colors
        .with_ticks(1.0, 0.15)  // Tick marks every unit
        .with_arrow_size(0.25)
        .with_resolution(15.0); // Higher resolution for smoother lines

    renderer.enable_axes(axes_config);

    println!("Rendering image with coordinate axes...");

    // Render the image
    let image = renderer.render(&cloud, &camera)?;

    // Save the image
    let output_path = "output.png";
    image.save(output_path)?;

    println!("Saved rendered image to: {}", output_path);
    println!("Image dimensions: {}x{}", image.width(), image.height());

    // Demonstrate camera manipulation and re-rendering
    println!("\nDemonstrating camera controls with axes...");

    // Orbit around the scene
    camera.orbit(std::f32::consts::PI / 4.0, 0.0)?; // 45 degrees yaw
    let image2 = renderer.render(&cloud, &camera)?;
    image2.save("output_orbited.png")?;
    println!("Saved orbited view to: output_orbited.png");

    // Zoom in
    camera.zoom(2.0)?; // 2x zoom
    let image3 = renderer.render(&cloud, &camera)?;
    image3.save("output_zoomed.png")?;
    println!("Saved zoomed view to: output_zoomed.png");

    // Change point size and background but keep axes
    renderer.set_point_size(8.0)?;
    renderer.set_background_color(Color::WHITE);
    let image4 = renderer.render(&cloud, &camera)?;
    image4.save("output_large_points.png")?;
    println!("Saved large points view to: output_large_points.png");

    // Show example without axes for comparison
    renderer.disable_axes();
    let image5 = renderer.render(&cloud, &camera)?;
    image5.save("output_no_axes.png")?;
    println!("Saved view without axes to: output_no_axes.png");

    println!("\nExample complete! Check the generated PNG files.");
    println!("All images include coordinate axes:");
    println!("  - Red axis = X direction");
    println!("  - Green axis = Y direction");
    println!("  - Blue axis = Z direction");
    println!("  - Tick marks show unit spacing");
    println!("  - Arrowheads and X/Y/Z labels indicate positive directions");

    // Now run the additional examples
    generate_spiral_scene()?;
    test_rendering_styles()?;

    Ok(())
}

/// Example showing programmatic scene generation
fn generate_spiral_scene() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nGenerating spiral scene...");

    let mut cloud = PointCloud::new();

    // Create a 3D spiral
    let num_points = 100;
    for i in 0..num_points {
        let t = i as f32 / num_points as f32;
        let angle = t * 8.0 * std::f32::consts::PI; // 4 full rotations
        let radius = 3.0 * t; // Expanding radius

        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = t * 6.0 - 3.0; // Height varies from -3 to +3

        // Color varies along the spiral (HSV-like)
        let hue = t * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        let color = Color::new(r, g, b);

        cloud.add_point_coords(x, y, z, color);
    }

    // Set up camera to view the spiral
    let mut camera = Camera::look_at(
        Vec3::new(8.0, 8.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
    );
    camera.set_fov_degrees(60.0)?;

    // Create renderer
    let mut renderer = ImageRenderer::new(1024, 768)?;
    renderer.set_background_color(Color::BLACK);
    renderer.set_point_size(3.0)?;

    // Render
    let image = renderer.render(&cloud, &camera)?;
    image.save("spiral.png")?;

    println!("Saved spiral scene to: spiral.png");

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

/// Example showing different rendering configurations
fn test_rendering_styles() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting different rendering styles...");

    // Create a simple test scene
    let mut cloud = PointCloud::new();

    // Grid of points
    for x in -2..=2 {
        for y in -2..=2 {
            for z in -1..=1 {
                let color = Color::new(
                    ((x + 2) * 50) as u8,
                    ((y + 2) * 50) as u8,
                    ((z + 1) * 100) as u8,
                );
                cloud.add_point_coords(x as f32, y as f32, z as f32, color);
            }
        }
    }

    let camera = Camera::look_at(
        Vec3::new(6.0, 6.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
    );

    // Test different point sizes
    for size in [1.0, 2.0, 4.0, 8.0] {
        let mut renderer = ImageRenderer::new(400, 400)?;
        renderer.set_point_size(size)?;
        renderer.set_background_color(Color::new(10, 10, 20));

        let image = renderer.render(&cloud, &camera)?;
        let filename = format!("test_size_{}.png", size as u32);
        image.save(&filename)?;
        println!("Saved {} with point size {}", filename, size);
    }

    // Test different backgrounds
    let backgrounds = [
        ("black", Color::BLACK),
        ("white", Color::WHITE),
        ("blue", Color::new(0, 50, 100)),
        ("purple", Color::new(50, 0, 100)),
    ];

    for (name, bg_color) in backgrounds {
        let mut renderer = ImageRenderer::new(400, 400)?;
        renderer.set_point_size(3.0)?;
        renderer.set_background_color(bg_color);

        let image = renderer.render(&cloud, &camera)?;
        let filename = format!("test_bg_{}.png", name);
        image.save(&filename)?;
        println!("Saved {} with {} background", filename, name);
    }

    Ok(())
}

// Uncomment to run additional examples
#[allow(dead_code)]
fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    generate_spiral_scene()?;
    test_rendering_styles()?;
    Ok(())
}