use altostratus::{PointCloud, Camera, AsciiRenderer, Renderer, Color, AxesConfig, CharacterSet};
use glam::Vec3;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Altostratus - ASCII Spiral Scene");
    println!("================================\n");
    
    let mut cloud = PointCloud::new();
    
    // Create a 3D spiral (same as image version)
    let num_points = 200; // Fewer points for ASCII to avoid clutter
    println!("Generating {} points in a spiral pattern...", num_points);
    
    for i in 0..num_points {
        let t = i as f32 / num_points as f32;
        let angle = t * 12.0 * std::f32::consts::PI; // 3 full rotations (reduced from 4)
        let radius = 2.5 * t; // Expanding radius (slightly smaller)
        
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = t * 4.0 - 2.0; // Height varies from -2 to +2 (reduced from -3 to +3)
        
        // Color varies along the spiral (HSV-like)
        let hue = t * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        let color = Color::new(r, g, b);
        
        cloud.add_point_coords(x, y, z, color);
    }
    
    println!("Created spiral with {} points", cloud.len());
    
    // Set up camera to view the spiral from multiple angles
    let viewpoints = [
        ("Isometric View", Vec3::new(6.0, 6.0, 6.0)),
        ("Side View", Vec3::new(8.0, 0.0, 0.0)),
        ("Top View", Vec3::new(0.0, 0.0, 8.0)),
        ("Front View", Vec3::new(0.0, 8.0, 0.0)),
    ];
    
    for (view_name, camera_pos) in viewpoints {
        println!("\n{}", "=".repeat(60));
        println!("{}", view_name);
        println!("{}", "=".repeat(60));
        
        let camera = Camera::look_at(camera_pos, Vec3::new(0.0, 0.0, 0.0));
        
        // Create ASCII renderer with dots character set
        let mut renderer = AsciiRenderer::new(60, 30)?;
        renderer.set_character_set(CharacterSet::Dots);
        
        // Configure axes for this view
        let axes_config = AxesConfig::new()
            .with_length(3.0)
            .with_ticks(1.0, 0.1)
            .with_resolution(20.0); // High resolution for smooth ASCII axes
        
        renderer.enable_axes(axes_config);
        
        println!("Camera: pos={:.1?}, looking at origin", camera_pos);
        println!("Character set: Dots ( .·•○● )");
        println!("Viewport: 60x30 characters\n");
        
        // Render the spiral
        let output = renderer.render(&cloud, &camera)?;
        
        // Add a border for better visualization
        println!("┌{}┐", "─".repeat(60));
        for (i, line) in output.lines().enumerate() {
            println!("│{}│{:2}", line, i);
        }
        println!("└{}┘", "─".repeat(60));
        
        println!("\nLegend:");
        println!("  · = Far points (light)");
        println!("  • = Medium distance"); 
        println!("  ○ = Closer points");
        println!("  ● = Very close points (dense)");
        println!("  Red axis = X, Green axis = Y, Blue axis = Z");
        
        // Add some spacing between views
        if view_name != "Front View" {
            println!("\nPress Enter to continue to next view...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("ASCII Spiral Rendering Complete!");
    println!("{}", "=".repeat(60));
    
    // Test different character sets with the isometric view
    println!("\nTesting different character sets (Isometric view):");
    
    let camera = Camera::look_at(Vec3::new(6.0, 6.0, 6.0), Vec3::new(0.0, 0.0, 0.0));
    
    let character_sets = [
        ("Standard", CharacterSet::Standard),
        ("Blocks", CharacterSet::Blocks),
        ("Custom Dense", CharacterSet::Custom(vec![' ', '░', '▒', '▓', '█'])),
    ];
    
    for (name, charset) in character_sets {
        println!("\n--- {} Character Set ---", name);
        
        let mut renderer = AsciiRenderer::new(50, 25)?;
        renderer.set_character_set(charset);
        renderer.enable_default_axes();
        
        let output = renderer.render(&cloud, &camera)?;
        
        println!("┌{}┐", "─".repeat(50));
        for line in output.lines() {
            println!("│{}│", line);
        }
        println!("└{}┘", "─".repeat(50));
    }
    
    println!("\nAlignment Analysis:");
    println!("- Look for proper spiral curvature in all views");
    println!("- Check that axes are orthogonal and properly aligned");
    println!("- Verify that the spiral expands outward and rises in Z");
    println!("- Side view should show clear Z progression");
    println!("- Top view should show XY spiral pattern");
    
    Ok(())
}

/// Simple HSV to RGB conversion (same as before)
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