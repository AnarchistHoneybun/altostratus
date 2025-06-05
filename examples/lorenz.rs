use altostratus::{PointCloud, Camera, ImageRenderer, Renderer, Color, AxesConfig};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Altostratus - Lorenz Attractor Visualization");
    println!("============================================");

    let mut cloud = PointCloud::new();

    // Lorenz system parameters (classic chaotic values)
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    // Initial conditions
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;

    // Integration parameters
    let dt = 0.005;
    let num_steps = 5000;
    
    println!("Simulating Lorenz attractor with {} time steps...", num_steps);
    println!("Parameters: σ={}, ρ={}, β ≈ {:.3}", sigma, rho, beta);

    for i in 0..num_steps {
        // Calculate derivatives (Lorenz equations)
        let dx_dt = sigma * (y - x);
        let dy_dt = x * (rho - z) - y;
        let dz_dt = x * y - beta * z;

        // Euler integration step
        x += dx_dt * dt;
        y += dy_dt * dt;
        z += dz_dt * dt;

        // Calculate trajectory progress for coloring
        let progress = i as f32 / num_steps as f32;
        
        // Color transitions: blue → cyan → green → yellow → red → magenta
        let color = if progress < 0.2 {
            // Blue to cyan
            let t = progress / 0.2;
            Color::new(0, (100.0 * t) as u8, 255)
        } else if progress < 0.4 {
            // Cyan to green  
            let t = (progress - 0.2) / 0.2;
            Color::new(0, 255, (255.0 * (1.0 - t)) as u8)
        } else if progress < 0.6 {
            // Green to yellow
            let t = (progress - 0.4) / 0.2;
            Color::new((255.0 * t) as u8, 255, 0)
        } else if progress < 0.8 {
            // Yellow to red
            let t = (progress - 0.6) / 0.2;
            Color::new(255, (255.0 * (1.0 - t)) as u8, 0)
        } else {
            // Red to magenta
            let t = (progress - 0.8) / 0.2;
            Color::new(255, 0, (255.0 * t) as u8)
        };

        // Calculate system velocity magnitude for point sizing
        let velocity_magnitude = f32::sqrt(dx_dt * dx_dt + dy_dt * dy_dt + dz_dt * dz_dt);
        
        // Only add every few points to avoid overcrowding but vary density
        let sample_rate = if velocity_magnitude < 15.0 { 1 } else { 2 };
        
        if i % sample_rate == 0 {
            cloud.add_point_coords(x, y, z, color);
        }
    }

    println!("Generated {} points tracing the chaotic attractor", cloud.len());
    println!("Final position: ({:.2}, {:.2}, {:.2})", x, y, z);

    // Set up camera to capture the full attractor
    let mut camera = Camera::look_at(
        Vec3::new(50.0, -10.0, 30.0),  // Good vantage point for Lorenz attractor
        Vec3::new(0.0, 0.0, 25.0),     // Look at center of attractor
    );
    camera.set_fov_degrees(70.0)?;

    println!("Camera positioned for optimal attractor viewing");

    // Create renderer with appropriate settings
    let mut renderer = ImageRenderer::new(1200, 900)?;
    renderer.set_background_color(Color::new(5, 5, 15)); // Deep space blue
    renderer.set_point_size(1.5)?;

    // Configure coordinate axes to show the attractor's orientation
    let axes_config = AxesConfig::new()
        .with_length(40.0)  // Match attractor scale
        .with_colors(
            Color::new(200, 100, 100),   // Subtle red for X
            Color::new(100, 200, 100),   // Subtle green for Y  
            Color::new(100, 100, 200)    // Subtle blue for Z
        )
        .with_ticks(10.0, 0.5)  // Major ticks every 10 units
        .with_resolution(30.0); // Smooth axes

    renderer.enable_axes(axes_config);

    println!("Rendering main view (1200x900)...");

    // Main view - classic perspective
    let image = renderer.render(&cloud, &camera)?;
    image.save("lorenz_attractor.png")?;
    println!("✓ Saved main view to: lorenz_attractor.png");

    // Generate additional dramatic views
    println!("\nGenerating additional perspectives...");

    // Side view - shows the "butterfly wings" clearly
    camera.set_position(Vec3::new(80.0, 0.0, 25.0));
    let image_side = renderer.render(&cloud, &camera)?;
    image_side.save("lorenz_side.png")?;
    println!("✓ Saved side view to: lorenz_side.png");

    // Top view - shows the spiral structure
    camera.set_position(Vec3::new(0.0, 0.0, 100.0));
    let image_top = renderer.render(&cloud, &camera)?;
    image_top.save("lorenz_top.png")?;
    println!("✓ Saved top view to: lorenz_top.png");

    // Close-up with larger points and different background
    camera.set_position(Vec3::new(30.0, -25.0, 35.0));
    renderer.set_point_size(3.0)?;
    renderer.set_background_color(Color::new(0, 0, 0)); // Pure black
    let image_closeup = renderer.render(&cloud, &camera)?;
    image_closeup.save("lorenz_closeup.png")?;
    println!("✓ Saved close-up view to: lorenz_closeup.png");

    // Artistic view - dramatic angle with enhanced colors
    camera.set_position(Vec3::new(-60.0, 60.0, 15.0));
    renderer.set_point_size(2.0)?;
    renderer.set_background_color(Color::new(10, 0, 20)); // Deep purple
    let image_artistic = renderer.render(&cloud, &camera)?;
    image_artistic.save("lorenz_artistic.png")?;
    println!("✓ Saved artistic view to: lorenz_artistic.png");

    println!("\nLorenz attractor visualization complete!");
    println!("Generated files:");
    println!("  - lorenz_attractor.png (main perspective)");
    println!("  - lorenz_side.png (butterfly wings view)");
    println!("  - lorenz_top.png (spiral structure)");
    println!("  - lorenz_closeup.png (detailed view)");
    println!("  - lorenz_artistic.png (dramatic angle)");
    
    println!("  • Trajectory length: {:.1} time units", num_steps as f32 * dt);

    lorenz_with_rk4()?;

    Ok(())
}

/// Alternative implementation with different integration methods
#[allow(dead_code)]
fn lorenz_with_rk4() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Lorenz attractor with 4th-order Runge-Kutta integration...");
    
    let mut cloud = PointCloud::new();
    
    // Lorenz parameters
    let sigma = 10.0;
    let rho = 28.0; 
    let beta = 8.0 / 3.0;
    
    // State vector [x, y, z]
    let mut state = [1.0, 1.0, 1.0];
    let dt = 0.0018; // Smaller step for RK4
    let num_steps = 16000;
    
    for i in 0..num_steps {
        // RK4 integration step
        let k1 = lorenz_derivatives(state, sigma, rho, beta);
        
        let state_k2 = [
            state[0] + 0.5 * dt * k1[0],
            state[1] + 0.5 * dt * k1[1], 
            state[2] + 0.5 * dt * k1[2],
        ];
        let k2 = lorenz_derivatives(state_k2, sigma, rho, beta);
        
        let state_k3 = [
            state[0] + 0.5 * dt * k2[0],
            state[1] + 0.5 * dt * k2[1],
            state[2] + 0.5 * dt * k2[2], 
        ];
        let k3 = lorenz_derivatives(state_k3, sigma, rho, beta);
        
        let state_k4 = [
            state[0] + dt * k3[0],
            state[1] + dt * k3[1],
            state[2] + dt * k3[2],
        ];
        let k4 = lorenz_derivatives(state_k4, sigma, rho, beta);
        
        // Update state
        state[0] += dt / 6.0 * (k1[0] + 2.0*k2[0] + 2.0*k3[0] + k4[0]);
        state[1] += dt / 6.0 * (k1[1] + 2.0*k2[1] + 2.0*k3[1] + k4[1]);
        state[2] += dt / 6.0 * (k1[2] + 2.0*k2[2] + 2.0*k3[2] + k4[2]);
        
        // Enhanced coloring based on system behavior
        let progress = i as f32 / num_steps as f32;
        let z_normalized = (state[2] - 5.0) / 40.0; // Normalize Z to [0,1]
        
        // Color based on height and time
        let hue = (progress * 360.0 + z_normalized * 120.0) % 360.0;
        let color = hsv_to_rgb(hue, 0.8, 0.9);
        
        if i % 2 == 0 { // Sample every 3rd point
            cloud.add_point_coords(state[0], state[1], state[2], color);
        }
    }
    
    // Render high-quality version
    let mut camera = Camera::look_at(
        Vec3::new(-20.0, 40.0, -5.0),
        Vec3::new(0.0, 0.0, 25.0),
    );

    camera.set_fov_degrees(70.0)?;

    // let axes_config = AxesConfig::new()
    //     .with_length(40.0)  // Match attractor scale
    //     .with_colors(
    //         Color::new(200, 100, 100),   // Subtle red for X
    //         Color::new(100, 200, 100),   // Subtle green for Y  
    //         Color::new(100, 100, 200)    // Subtle blue for Z
    //     )
    //     .with_ticks(10.0, 0.5)  // Major ticks every 10 units
    //     .with_resolution(30.0); // Smooth axes
    
    let mut renderer = ImageRenderer::new(1600, 1200)?;
    renderer.set_background_color(Color::new(2, 2, 8));
    renderer.set_point_size(1.1)?;
    // renderer.enable_axes(axes_config);
    
    let image = renderer.render(&cloud, &camera)?;
    image.save("lorenz_rk4_hq.png")?;
    
    println!("✓ High-quality RK4 version saved to: lorenz_rk4_hq.png");
    
    Ok(())
}

fn lorenz_derivatives(state: [f32; 3], sigma: f32, rho: f32, beta: f32) -> [f32; 3] {
    [
        sigma * (state[1] - state[0]),
        state[0] * (rho - state[2]) - state[1],
        state[0] * state[1] - beta * state[2],
    ]
}

/// HSV to RGB color conversion for smooth color transitions
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
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

    Color::new(
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    )
}