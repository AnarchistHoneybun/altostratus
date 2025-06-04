# altostratus

*Altostratus is a middle-altitude cloud genus made up of water droplets, ice crystals, or a mixture of the two.*

A 3d plotting library for Rust with one purpose, to render points in a cartesian system. You might wonder why you'd rather not use something more general, with better code, better suited to this purpose. I do too.

## features

- display points in 3d space
- provide two rendering systems: image, and ascii (this is really broken right now)
- camera controls: zoom, pan, orbit, rotate

## basic usage

```rust
use altostratus::{PointCloud, Camera, ImageRenderer, Renderer, Color};
use glam::Vec3;

// Create a point cloud
let mut cloud = PointCloud::new();
cloud.add_point_coords(0.0, 0.0, 0.0, Color::RED);
cloud.add_point_coords(1.0, 1.0, 1.0, Color::BLUE);

// Set up camera
let camera = Camera::look_at(Vec3::new(2.0, 2.0, 3.0), Vec3::ZERO);

// Render to image
let mut renderer = ImageRenderer::new(800, 600)?;
renderer.enable_default_axes();
let image = renderer.render(&cloud, &camera)?;
image.save("plot.png")?;
```

## quick start

```bash
git clone https://github.com/username/altostratus
cd altostratus

# Run tests
cargo test

# Try the examples
cargo run --example basic_rendering
cargo run --example spiral           
cargo run --example axes_test       
cargo run --example ascii_rendering 
cargo run --example simple_ascii
```

## architecture

Altostratus follows a **trait-based renderer abstraction** pattern:

```rust
trait Renderer {
    type Output;
    fn render(&mut self, points: &PointCloud, camera: &Camera) -> Result<Self::Output>;
}

// ImageRenderer::Output = image::RgbImage
// AsciiRenderer::Output = String
```

**Core Components:**
- **Data Layer**: `PointCloud`, `Point3D`, `Color`
- **Camera System**: Mutable 3D camera with projection matrices
- **Math Layer**: `Projector`, `FrustumCuller`, `DepthBuffer` 
- **Rendering**: `ImageRenderer`, `AsciiRenderer` with shared infrastructure
- **Visualization**: `Axes` system for coordinate reference

## progress & todo

###  completed features

- [x] **Core Data Structures**
  - [x] Point3D with position and color
  - [x] PointCloud container with dynamic point addition
  - [x] RGB color system with constants
  - [x] Comprehensive error handling

- [x] **Camera System** 
  - [x] Mutable camera properties (position, target, FOV, etc.)
  - [x] Camera controls (orbit, zoom, pan, rotate, translate)
  - [x] Automatic aspect ratio handling
  - [x] Bounding box auto-framing

- [x] **3D Mathematics & Rendering**
  - [x] Perspective projection with proper matrices
  - [x] Frustum culling for performance
  - [x] Depth buffer with Z-testing
  - [x] Screen coordinate transformation
  - [x] Renderer trait abstraction

- [x] **Image Renderer**
  - [x] PNG output via `image` crate
  - [x] Multiple point drawing styles (pixel, square, circle)
  - [x] Configurable point sizes and background colors
  - [x] Proper depth sorting and anti-aliasing
  - [x] High-quality visual output

- [x] **Coordinate Axes System**
  - [x] Configurable X/Y/Z axes with custom colors
  - [x] Tick marks with spacing control
  - [x] Arrowheads and geometric labels (X, Y, Z)
  - [x] Integration with both renderers

- [x] **ASCII Renderer (Basic)**
  - [x] Multiple character sets (Standard, Blocks, Dots, Custom)
  - [x] Depth-to-character mapping
  - [x] Terminal coordinate handling
  - [x] ANSI color support
  - [x] Advanced renderer with borders and info

- [x] **Testing & Examples**
  - [x] 100+ comprehensive tests
  - [x] Multiple example programs
  - [x] Debug utilities and error handling

### in progress

- [ ] **ASCII Renderer Improvements**
  - [ ] Fix alignment and aspect ratio issues
  - [ ] Improve character density mapping
  - [ ] Better terminal character handling
  - [ ] Enhanced coordinate axes rendering

### planned features

- [ ] **Performance Optimizations**
  - [ ] Spatial indexing (octree) for large datasets
  - [ ] SIMD optimizations for point processing
  - [ ] Memory-mapped file support for huge datasets
  - [ ] Level-of-detail (LOD) rendering

- [ ] **Advanced Rendering**
  - [ ] Anti-aliasing for image renderer
  - [ ] Gradient backgrounds
  - [ ] Point sprites and custom shapes
  - [ ] Line and surface rendering
  - [ ] Lighting and shading

- [ ] **Data Import/Export**
  - [ ] CSV/JSON data loading
  - [ ] Multiple coordinate systems

- [ ] **Interactive Features**
  - [ ] Real-time camera manipulation
  - [ ] Mouse/keyboard controls
  - [ ] Animation and keyframe system
  - [ ] Interactive terminal interface

## contributing

contributions are welcome, issues, and feature requests are encouraged. I'm writing this just to scratch the itch of not having to put images on my website, but have close to zero experience with rendering etc. so if you have an idea that improves this, open up an issue to make me aware of it. 