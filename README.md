# wgpu Part 2: Textured Cubes, Camera and Instancing

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![wgpu](https://img.shields.io/badge/wgpu-30.0.1-blue)
![winit](https://img.shields.io/badge/winit-0.30.13-green)
![glam](https://img.shields.io/badge/glam-0.34.1-purple)

A real-time 3D scene built with [wgpu](https://wgpu.rs): a textured, lit cube drawn 10,000 times in a single draw call, with an orbit camera you control with the mouse.

This is the second part of a step-by-step wgpu series. Part 1 drew a triangle with its data hardcoded in the shader. Part 2 replaces that with the building blocks of a real renderer.

![Part 2 screenshot](docs/part2.png)

## Features

- Textured cube with per-face normals and simple diffuse lighting
- 10,000 cubes (a 100 × 100 grid) rendered with GPU instancing in one draw call
- Perspective camera with mouse orbit controls
- Depth buffer, recreated on window resize
- Camera aspect ratio that follows the window
- Slowly rotating scene
- FPS counter printed to the terminal

## What you'll learn

- Vertex buffers and index buffers, and how to describe their layout to the GPU
- Loading an image and uploading it as a texture with a sampler
- Bind groups and bind group layouts
- Uniform buffers and updating them every frame
- View and projection matrices
- Why a depth buffer is needed
- Instancing: many copies of one mesh, each with its own transform
- Using surface normals for basic lighting
- Handling raw mouse input in winit

## Getting started

### Prerequisites

- [Rust](https://rustup.rs) (edition 2024, so a recent stable toolchain)
- A GPU with a Vulkan, Metal or DirectX 12 driver
- A texture at `assets/crate.png`. It is embedded at compile time, so the build fails without it

### Run

```bash
git clone <this-repo-url>
cd <repo-folder>
cargo run --release
```

Use `--release`. Drawing 10,000 cubes and updating the camera every frame is much faster with optimizations on.

### Controls

| Input | Action |
|---|---|
| Hold left mouse button and move the mouse | Orbit the camera around the scene |
| Close the window | Quit |

## Project structure

| Path | Responsibility |
|---|---|
| `src/main.rs` | Window and event loop. Forwards window events and raw mouse motion to `State` |
| `src/state.rs` | Meshes, texture, camera, uniforms, instances, depth buffer, `update` and `render` |
| `src/shader.wgsl` | Vertex shader (transforms and normals) and fragment shader (texture and lighting) |
| `assets/crate.png` | Cube texture |

## How it works

### Mesh
The cube has 24 vertices, four per face, because each face needs its own normal and texture coordinates. An index buffer of 36 entries builds two triangles per face from those vertices. Every face is wound counter-clockwise when seen from outside, which matches the pipeline's front-face and culling settings.

### Vertex layout
Each vertex stores a position, texture coordinates and a normal. A layout description tells wgpu where each field starts and which shader input it feeds.

### Texture
The image is decoded to RGBA, uploaded to the GPU as an sRGB texture and read through a linear-filtering sampler. The texture and sampler are in bind group 0 and used by the fragment stage.

### Camera and uniforms
The camera has a position, a target, an up direction, a field of view and near and far planes. Every frame its combined view and projection matrix is written into a single uniform buffer in bind group 1, used by the vertex stage. When the window resizes, the aspect ratio is updated so the image doesn't stretch.

### Mouse orbit
Left-button state comes from window events. Mouse movement comes from raw device events. While the button is held, movement changes yaw and pitch, and the camera is placed on a sphere around the scene. Pitch is clamped so the camera can't flip over the top.

### Depth buffer
Without depth testing, faces are drawn in submission order and far ones can cover near ones. A depth texture is cleared every frame, the pipeline keeps the nearest fragment, and the texture is recreated whenever the window size changes.

### Instancing
A second vertex buffer holds one model matrix per cube. It advances once per instance rather than once per vertex. A matrix is too big for a single vertex attribute, so it is passed as four 4-component attributes and reassembled in the shader. The whole grid is then drawn with one indexed draw call. Instance data is uploaded once. Only the camera uniform changes per frame.

### Shading
The vertex stage builds the model matrix, transforms the position to clip space and rotates the normal into world space. The fragment stage samples the texture and scales it by a diffuse term (how directly the face points toward a fixed light) plus a small ambient amount, so faces pointing away from the light are dim, not black.

### Bind groups at a glance

| Group | Binding | Resource | Used by |
|---|---|---|---|
| 0 | 0 | Texture | Fragment |
| 0 | 1 | Sampler | Fragment |
| 1 | 0 | Uniform buffer (camera matrix) | Vertex |

## Experiments

- Change the grid size and spacing and compare the FPS.
- Move the light direction and watch the shading change.
- Swap in your own texture.
- Set the ambient light to zero and see which faces go black.
- Show the normals as colors instead of the texture.
- Remove the depth state from the pipeline and see what breaks.

## Tech stack

| Crate | Version | Purpose |
|---|---|---|
| `wgpu` | 30.0.1 | GPU access |
| `winit` | 0.30.13 | Window and events |
| `glam` | 0.34.1 | Vector, matrix and quaternion math |
| `bytemuck` | 1.25.0 | Safely casting structs to bytes for GPU buffers |
| `image` | 0.25.10 | Decoding the texture |
| `pollster` | 0.4.0 | Blocks on wgpu's async setup |

## Notes

- Lighting assumes instances are only rotated and translated, with no non-uniform scaling.
- The FPS counter prints to the terminal once per second.
