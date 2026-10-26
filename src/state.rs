// state.rs holds everything that talks to the GPU.

use std::sync::Arc;

use winit::{
    dpi::PhysicalSize,
    event_loop::OwnedDisplayHandle,
    window::Window,
    event::{ElementState, MouseButton, WindowEvent}
};

use wgpu::util::DeviceExt;
use glam::camera::rh::{proj::directx, view};
use glam::{Mat4, Vec3, Quat};


struct Instance {
    position: Vec3,
    rotation: Quat,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: Mat4,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation)
        }
    }
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }

    }
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor{
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection(&self) -> Mat4 {
        let view = view::look_at_mat4(self.eye, self.target, self.up);
        let proj = directx::perspective(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: Mat4,
}

impl Uniforms {
    fn new() -> Self {
        Self { view_proj: Mat4::IDENTITY }
    }

    fn update(&mut self, camera: &Camera, angle: f32) {
        let model = Mat4::from_rotation_y(angle);
        self.view_proj = camera.build_view_projection() * model;
    }

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }
}

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
     // Front face, bottom left, pointing along +Z.
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], normal: [ 0.0,  0.0,  1.0] },
    // Front face, bottom right, same direction.
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], normal: [ 0.0,  0.0,  1.0] },
    // Front face, top right, same direction.
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], normal: [ 0.0,  0.0,  1.0] },
    // Front face, top left, same direction.
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], normal: [ 0.0,  0.0,  1.0] },

    // Back face, bottom left, pointing along -Z.
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [ 0.0,  0.0, -1.0] },
    // Back face, bottom right.
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [ 0.0,  0.0, -1.0] },
    // Back face, top right.
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [ 0.0,  0.0, -1.0] },
    // Back face, top left.
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [ 0.0,  0.0, -1.0] },

    // Right face, bottom left, pointing along +X.
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], normal: [ 1.0,  0.0,  0.0] },
    // Right face, bottom right.
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [ 1.0,  0.0,  0.0] },
    // Right face, top right.
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [ 1.0,  0.0,  0.0] },
    // Right face, top left.
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], normal: [ 1.0,  0.0,  0.0] },

    // Left face, bottom left, pointing along -X.
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [-1.0,  0.0,  0.0] },
    // Left face, bottom right.
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], normal: [-1.0,  0.0,  0.0] },
    // Left face, top right.
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], normal: [-1.0,  0.0,  0.0] },
    // Left face, top left.
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [-1.0,  0.0,  0.0] },

    // Top face, bottom left, pointing along +Y.
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0], normal: [ 0.0,  1.0,  0.0] },
    // Top face, bottom right.
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0], normal: [ 0.0,  1.0,  0.0] },
    // Top face, top right.
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], normal: [ 0.0,  1.0,  0.0] },
    // Top face, top left.
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], normal: [ 0.0,  1.0,  0.0] },

    // Bottom face, bottom left, pointing along -Y.
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], normal: [ 0.0, -1.0,  0.0] },
    // Bottom face, bottom right.
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], normal: [ 0.0, -1.0,  0.0] },
    // Bottom face, top right.
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], normal: [ 0.0, -1.0,  0.0] },
    // Bottom face, top left.
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], normal: [ 0.0, -1.0,  0.0] },
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    // Front face, two triangles sharing corners 0 and 2.
     0,  1,  2,   0,  2,  3,
    // Back face, the same pattern shifted by four.
     4,  5,  6,   4,  6,  7,
    // Right face.
     8,  9, 10,   8, 10, 11,
    // Left face.
    12, 13, 14,  12, 14, 15,
    // Top face.
    16, 17, 18,  16, 18, 19,
    // Bottom face.
    20, 21, 22,  20, 22, 23,
];

pub struct State {
    window: Arc<Window>,
    #[allow(dead_code)]
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    texture_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    last_frame: std::time::Instant,
    rotation: f32,
    mouse_pressed: bool,
    yaw: f32,
    pitch: f32,
    camera: Camera,
    depth_view: wgpu::TextureView,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    fps_timer: f32,
    fps_frames: u32,
}

impl State {
    pub async fn new(window: Arc<Window>, display: OwnedDisplayHandle) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));

        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create the surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("failed to create the device");

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("the surface is not supported by this adapter");
        config.format = format;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        let img_bytes = include_bytes!("../assets/crate.png");
        let img = image::load_from_memory(img_bytes).expect("failed to decode the texture");
        let rgba = img.to_rgba8();
        let (img_w, img_h) = rgba.dimensions();

        let texture_size = wgpu::Extent3d {
            width: img_w,
            height: img_h,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("crate texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * img_w),
                rows_per_image: Some(img_h),
            },
            texture_size
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            label: Some("crate sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture{
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float{filterable: true},
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering
                        ),
                        count: None,
                    },
                ]
            });

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor{
                label: Some("texture bind group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ]
            });

        let camera = Camera {
            eye: Vec3::new(0.0, 1.0, 2.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 1000.0,
        };

        let mut uniforms = Uniforms::new();
        uniforms.update(&camera, 0.0);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
                label: Some("uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        const GRID: i32 = 100;
        const SPACING: f32 = 3.0;

        let instances: Vec<Instance> = (0..GRID)
            .flat_map(|z| {
                (0..GRID).map(move |x| Instance {
                    position: Vec3::new(
                        (x - GRID/2) as f32 * SPACING,
                        0.0,
                        (z - GRID/2) as f32 * SPACING,
                    ),
                    rotation: Quat::from_axis_angle(Vec3::Y, ((x*7+z*13) as f32).to_radians()
                    ),
                })
            })
            .collect();

        let instance_data: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
        let num_instances = instance_data.len() as u32;

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_view = create_depth_view(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(Vertex::desc()),
                    Some(InstanceRaw::desc()),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
            texture_bind_group,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            last_frame: std::time::Instant::now(),
            rotation: 0.0,
            mouse_pressed: false,
            yaw: 0.0,
            pitch: 0.3,
            camera,
            depth_view,
            instance_buffer,
            num_instances,
            fps_timer: 0.0,
            fps_frames: 0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        self.depth_view = create_depth_view(&self.device, &self.config);

    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn mouse_move(&mut self, dx: f64, dy: f64) {
        if !self.mouse_pressed {
            return;
        }

        let sensitivity = 0.005;
        self.yaw -= dx as f32 * sensitivity;
        self.pitch += dy as f32 * sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.fps_timer += dt;
        self.fps_frames += 1;

        if self.fps_timer >= 1.0 {
            println!("FPS: {}", self.fps_frames);
            self.fps_frames = 0;
            self.fps_timer = 0.0;
        }


        self.rotation += dt * 0.4;

        let radius = 60.0;
        self.camera.eye = Vec3::new(
            radius * self.yaw.sin() * self.pitch.cos(),
            radius * self.pitch.sin(),
            radius * self.yaw.cos() * self.pitch.cos(),
        );

        self.uniforms.update(&self.camera, self.rotation);

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.texture_bind_group, &[]);
            pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
    }
}
