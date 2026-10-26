# wgpu Part 1: Your First Triangle

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![wgpu](https://img.shields.io/badge/wgpu-30.0.1-blue)
![winit](https://img.shields.io/badge/winit-0.30.13-green)

A minimal Rust project that opens a window and draws a colored triangle on the GPU using [wgpu](https://wgpu.rs) and [winit](https://github.com/rust-windowing/winit).

This is the first part of a step-by-step wgpu series. It covers the setup every GPU program needs, with as little code as possible around it, so you can see what each piece does.

![Part 1 screenshot](docs/part1.png)

## Features

- Cross-platform window and event loop (winit)
- Full wgpu initialization: instance, surface, adapter, device and queue
- A render pipeline with a WGSL vertex and fragment shader
- A triangle with red, green and blue corners, blended smoothly by the GPU
- Window resizing and surface-loss handling
- No vertex buffers or textures. All triangle data lives in the shader

## What you'll learn

- How wgpu objects relate to each other and in what order they are created
- How to configure a window surface (format, size, vsync)
- What a render pipeline is and which settings matter
- How vertex shaders and fragment shaders pass data to each other
- How the rasterizer interpolates values across a triangle
- What winding order and back-face culling do
- How a frame is drawn: acquire, encode, submit, present

## Getting started

### Prerequisites

- [Rust](https://rustup.rs) (edition 2024, so a recent stable toolchain)
- A GPU with a Vulkan, Metal or DirectX 12 driver

### Run

```bash
git clone <this-repo-url>
cd <repo-folder>
cargo run
```

You should see a dark window with a triangle that is red at the top, green at the bottom left and blue at the bottom right.

## Project structure

| File | Responsibility |
|---|---|
| `src/main.rs` | Creates the window and runs the event loop. Forwards events to `State` |
| `src/state.rs` | Owns every GPU object. Handles setup, resizing and drawing a frame |
| `src/shader.wgsl` | Vertex shader (positions and colors) and fragment shader (outputs the color) |

## How it works

1. **Window.** `main.rs` creates the window and requests a redraw after every frame, so the app renders continuously.
2. **GPU setup.** `State::new` builds the instance, creates a surface from the window, picks an adapter that can draw to it, and opens a device and queue. It then configures the surface with an sRGB format and vsync.
3. **Shader.** The vertex shader runs three times. It uses the built-in vertex index to pick a position and a color from small arrays. Positions are in clip space, where x and y go from -1 to 1 and (0, 0) is the window center.
4. **Interpolation.** The vertex shader passes the color to the fragment shader. For each pixel the triangle covers, the GPU blends the three corner colors by distance to each corner.
5. **Pipeline.** The pipeline links the shader to fixed settings: triangle list topology, counter-clockwise front faces, back-face culling and no blending.
6. **Frame.** Each frame acquires the next surface texture, records a render pass that clears the screen and draws 3 vertices, then submits the commands and presents the result.
7. **Resize.** Zero-sized windows (minimized) are ignored. Otherwise the surface is reconfigured with the new size.

## Experiments

- Change the clear color.
- Change the colors in the shader and watch the gradient change.
- Reverse the vertex order. The triangle disappears because of back-face culling.
- Turn culling off and it comes back.
- Invert the colors in the fragment shader.

## Tech stack

| Crate | Version | Purpose |
|---|---|---|
| `wgpu` | 30.0.1 | GPU access |
| `winit` | 0.30.13 | Window and events |
| `pollster` | 0.4.0 | Blocks on wgpu's async setup |

The repository's `Cargo.toml` is shared by all parts, so it also lists crates that this part does not use.

## Next

**Part 2** moves the data out of the shader into GPU buffers and draws a textured, lit, instanced cube scene with a camera.
