#![allow(unused_imports)]
use tracing::{info, warn, error};
use std::iter;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = 
        wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3
        ];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;        

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.5, 0.0] },  
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 0.5, 0.5] },
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.0, 0.0, 1.0] },

    // second figure
    Vertex { position: [-0.4, 0.6, 0.0], color: [0.5, 0.0, 0.5] }, // A - 5
    Vertex { position: [-0.4, -0.8, 0.0], color: [0.5, 0.0, 0.5] }, // B - 6
    Vertex { position: [-0.2, -0.8, 0.0], color: [0.5, 0.0, 0.5] }, // C - 7
    Vertex { position: [-0.2, 0.0, 0.0], color: [0.5, 0.0, 0.5] }, // D - 8
    Vertex { position: [0.2, -0.8, 0.0], color: [0.5, 0.0, 0.5] }, // E - 9
    Vertex { position: [0.4, -0.8, 0.0], color: [0.5, 0.0, 0.5] }, // F - 10
    Vertex { position: [0.0, 0.0, 0.0], color: [0.5, 0.0, 0.5] }, // G - 11
    Vertex { position: [-0.2, 0.4, 0.0], color: [0.5, 0.0, 0.5] }, // H - 12
    Vertex { position: [0.0, 0.4, 0.0], color: [0.5, 0.0, 0.5] }, // I - 13
    Vertex { position: [0.2, 0.3, 0.0], color: [0.5, 0.0, 0.5] }, // J - 14
    Vertex { position: [0.2, 0.1, 0.0], color: [0.5, 0.0, 0.5] }, // K - 15
    Vertex { position: [0.2, 0.6, 0.0], color: [0.5, 0.0, 0.5] }, // L - 16
    Vertex { position: [0.4, 0.4, 0.0], color: [0.5, 0.0, 0.5] }, // M - 17
    Vertex { position: [-0.2, -0.2, 0.0], color: [0.5, 0.0, 0.5] }, // N - 18
    Vertex { position: [0.0, -0.2, 0.0], color: [0.5, 0.0, 0.5] }, // O - 19
    Vertex { position: [0.2, -0.2, 0.0], color: [0.5, 0.0, 0.5] }, // P - 20
    Vertex { position: [0.4, 0.0, 0.0], color: [0.5, 0.0, 0.5] }, // Q - 21
    Vertex { position: [-0.2, 0.6, 0.0], color: [0.5, 0.0, 0.5] }, // R - 22
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,

    //second figure
    22, 5, 6,
    22, 6, 7,
    22, 12, 17,
    22, 17, 16,
    17, 13, 14,
    17, 14, 15,
    17, 15, 21,
    21, 15, 11,
    21, 8, 18,
    21, 18, 20,
    18, 9, 10,
    18, 10, 19,
];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    window: Window,
    render_pipeline_array: [wgpu::RenderPipeline; 2],
    pipeline_toggle: usize,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indecies_ranges_array: [std::ops::Range<u32>; 2],
}

impl State {
    async fn new(window: Window) -> Self {
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap(); 

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let shader_challenge = shader; 

        let render_pipeline_layout_challenge =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout Challenge"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline_challenge = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline Challenge"),
            layout: Some(&render_pipeline_layout_challenge),
            vertex: wgpu::VertexState {
                module: &shader_challenge,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_challenge,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_pipeline_array = [render_pipeline, render_pipeline_challenge];

        let pipeline_toggle = 0;

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let indecies_ranges_array = [0..9, 9..INDICES.len() as u32];
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            window,
            render_pipeline_array,
            pipeline_toggle,
            vertex_buffer,
            index_buffer,
            indecies_ranges_array,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let red = position.x as f64 / self.size.width as f64;
                let green = position.y as f64 / self.size.height as f64;
                
                self.clear_color = wgpu::Color {
                    r: red,
                    g: green,
                    b: 0.3,
                    a: 1.0,
                };    
                
                return true;
            },
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                },
                ..
            } => {
                self.pipeline_toggle = (self.pipeline_toggle + 1) % self.render_pipeline_array.len();
                return true;
            }
            _ => return false
        }        
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let render_pipeline = &self.render_pipeline_array[self.pipeline_toggle];
            let indecies_range = &self.indecies_ranges_array[self.pipeline_toggle];

            render_pass.set_pipeline(render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(indecies_range.clone(), 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1800, 1600));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
