use altostratus::{PointCloud, Camera, AsciiRenderer, AdvancedAsciiRenderer, Renderer, Color, AxesConfig, CharacterSet};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Altostratus ASCII Renderer Examples");
    println!("===================================");

    // Run all ASCII rendering examples
    basic_ascii_example()?;
    println!("\n\n\n");

    character_sets_example()?;
    println!("\n\n\n");

    axes_example()?;
    println!("\n\n\n");

    color_example()?;
    println!("\n\n\n");

    advanced_renderer_example()?;

    Ok(())
}

/// Basic ASCII rendering example
fn basic_ascii_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Basic ASCII Rendering");
    println!("-----------------------");

    // Create a simple point cloud
    let mut cloud = PointCloud::new();

    // Add some points in a simple pattern
    cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);    // Center
    cloud.add_point_coords(1.0, 0.0, 0.0, Color::RED);      // Right
    cloud.add_point_coords(-1.0, 0.0, 0.0, Color::GREEN);   // Left
    cloud.add_point_coords(0.0, 1.0, 0.0, Color::BLUE);     // Up
    cloud.add_point_coords(0.0, -1.0, 0.0, Color::RED);  // Down
    cloud.add_point_coords(0.0, 0.0, 1.0, Color::GREEN);     // Forward

    // Position camera
    let camera = Camera::look_at(
        Vec3::new(3.0, 2.0, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
    );

    // Create ASCII renderer
    let mut renderer = AsciiRenderer::new(40, 20)?;

    println!("Rendering basic point cloud (40x20 characters):");
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    Ok(())
}

/// Different character sets example
fn character_sets_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Different Character Sets");
    println!("---------------------------");

    // Create a cube of points for better depth visualization
    let mut cloud = PointCloud::new();

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let color = Color::new(
                    ((x + 1) * 100) as u8,
                    ((y + 1) * 100) as u8,
                    ((z + 1) * 100) as u8,
                );
                cloud.add_point_coords(x as f32, y as f32, z as f32, color);
            }
        }
    }

    let camera = Camera::look_at(
        Vec3::new(4.0, 3.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
    );

    // Test different character sets
    let character_sets = [
        ("Standard", CharacterSet::Standard),
        ("Blocks", CharacterSet::Blocks),
        ("Dots", CharacterSet::Dots),
        ("Custom", CharacterSet::Custom(vec![' ', '.', 'o', 'O', '@'])),
    ];

    for (name, charset) in character_sets {
        println!("\n{} character set:", name);
        let mut renderer = AsciiRenderer::new(30, 15)?;
        renderer.set_character_set(charset);

        let output = renderer.render(&cloud, &camera)?;
        println!("{}", output);
    }

    Ok(())
}

/// Coordinate axes example
fn axes_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Coordinate Axes");
    println!("------------------");

    // Create a minimal point cloud
    let mut cloud = PointCloud::new();
    cloud.add_point_coords(0.0, 0.0, 0.0, Color::WHITE);
    cloud.add_point_coords(2.0, 1.0, 1.0, Color::RED);

    let camera = Camera::look_at(
        Vec3::new(5.0, 4.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
    );

    // Renderer without axes
    println!("Without axes:");
    let mut renderer = AsciiRenderer::new(35, 18)?;
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    // Renderer with default axes
    println!("\nWith coordinate axes:");
    renderer.enable_default_axes();
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    // Custom axes configuration
    println!("\nWith custom axes (tighter spacing):");
    let custom_axes = AxesConfig::new()
        .with_length(3.0)
        .with_ticks(0.5, 0.1)  // Closer tick marks
        .with_resolution(25.0); // Higher resolution for ASCII

    renderer.set_axes_config(Some(custom_axes));
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    Ok(())
}

/// Color output example (if terminal supports it)
fn color_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Color Output (ANSI colors)");
    println!("------------------------------");

    // Create a colorful scene
    let mut cloud = PointCloud::new();

    // Rainbow points
    let colors = [
        Color::new(255, 0, 0),     // Red
        Color::new(255, 127, 0),   // Orange
        Color::new(255, 255, 0),   // Yellow
        Color::new(0, 255, 0),     // Green
        Color::new(0, 0, 255),     // Blue
        Color::new(75, 0, 130),    // Indigo
        Color::new(148, 0, 211),   // Violet
    ];

    for (i, &color) in colors.iter().enumerate() {
        let angle = (i as f32 / colors.len() as f32) * 2.0 * std::f32::consts::PI;
        let x = 2.0 * angle.cos();
        let y = 2.0 * angle.sin();
        cloud.add_point_coords(x, y, 0.0, color);
    }

    let camera = Camera::look_at(
        Vec3::new(0.0, 0.0, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
    );

    // Render with colors
    let mut renderer = AsciiRenderer::with_color(30, 15)?;
    renderer.set_character_set(CharacterSet::Standard);

    println!("Colorful ASCII rendering (if your terminal supports ANSI colors):");
    let output = renderer.render(&cloud, &camera)?;
    println!("{}", output);

    println!("\nNote: If you see escape codes like \\x1b[38;5;196m, your terminal doesn't support ANSI colors.");

    Ok(())
}

/// Advanced renderer with border and info
fn advanced_renderer_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n5. Advanced ASCII Renderer");
    println!("--------------------------");

    // Create a spiral-like pattern
    let mut cloud = PointCloud::new();

    for i in 0..20 {
        let t = i as f32 / 10.0;
        let angle = t * std::f32::consts::PI;
        let x = t * angle.cos();
        let y = t * angle.sin();
        let z = t * 0.5;

        let color = Color::new(
            (127.0 + 127.0 * (t * 2.0).sin()) as u8,
            (127.0 + 127.0 * (t * 3.0).cos()) as u8,
            200,
        );

        cloud.add_point_coords(x, y, z, color);
    }

    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 4.0),
        Vec3::new(0.0, 0.0, 1.0),
    );

    // Advanced renderer with all features
    let mut advanced_renderer = AdvancedAsciiRenderer::new(35, 15)?;
    advanced_renderer.set_border(true, '#');
    advanced_renderer.set_info(true);

    // Configure the base renderer
    advanced_renderer.base_mut().set_character_set(CharacterSet::Blocks);
    advanced_renderer.base_mut().enable_default_axes();

    println!("Advanced renderer with border, info, and axes:");
    let output = advanced_renderer.render(&cloud, &camera)?;
    println!("{}", output);

    Ok(())
}

/// Interactive example for testing different viewpoints
#[allow(dead_code)]
fn interactive_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6. Interactive Demo");
    println!("-------------------");
    println!("This would be an interactive terminal app, but for now just shows different views:");

    // Create a 3D scene
    let mut cloud = PointCloud::new();

    // Create a simple house-like structure
    let house_points = [
        // Base square
        (-1.0, -1.0, 0.0), (1.0, -1.0, 0.0), (1.0, 1.0, 0.0), (-1.0, 1.0, 0.0),
        // Top square
        (-1.0, -1.0, 2.0), (1.0, -1.0, 2.0), (1.0, 1.0, 2.0), (-1.0, 1.0, 2.0),
        // Roof peak
        (0.0, 0.0, 3.0),
    ];

    for (i, &(x, y, z)) in house_points.iter().enumerate() {
        let color = if i < 4 {
            Color::new(139, 69, 19) // Brown base
        } else if i < 8 {
            Color::new(169, 169, 169) // Gray walls
        } else {
            Color::new(220, 20, 60) // Red roof
        };
        cloud.add_point_coords(x, y, z, color);
    }

    let viewpoints = [
        ("Front view", Vec3::new(0.0, -6.0, 1.5)),
        ("Side view", Vec3::new(6.0, 0.0, 1.5)),
        ("Top view", Vec3::new(0.0, 0.0, 8.0)),
        ("Isometric", Vec3::new(4.0, -4.0, 4.0)),
    ];

    let mut renderer = AsciiRenderer::new(25, 12)?;
    renderer.enable_default_axes();
    renderer.set_character_set(CharacterSet::Blocks);

    for (name, camera_pos) in viewpoints {
        println!("\n{}:", name);
        let camera = Camera::look_at(camera_pos, Vec3::new(0.0, 0.0, 1.5));
        let output = renderer.render(&cloud, &camera)?;
        println!("{}", output);
    }

    Ok(())
}

// You can uncomment this line to run the interactive demo
// interactive_demo()?;