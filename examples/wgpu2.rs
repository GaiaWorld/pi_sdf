use parry2d::na::{Orthographic3, Matrix4, Vector3};
use pi_sdf::{utils::FontFace, glyphy_draw2::get_char_arc, glyphy::vertex::{GlyphInfo, add_glyph_vertices}};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

use env_logger::Env;



use pi_wgpu as wgpu;
use wgpu::{util::DeviceExt, BlendState, ColorTargetState};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let subscriber = Subscriber::builder().with_max_level(Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let window_size = window.inner_size();
    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../source/glyphy.vs").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    // Load the shaders from disk
    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../source/glyphy.fs").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    // println!("vs: {:?}", vs);
    // println!("fs: {:?}", fs);
    let buffer = std::fs::read("./source/msyh.ttf").unwrap();
    let mut ft_face = FontFace::new(buffer);

    let mut gi = GlyphInfo::new();
    let arcs = get_char_arc(&mut gi, &mut ft_face, '魔', None);

    let verties = add_glyph_vertices(&gi, None, None);
    let tex_data = arcs.tex_data;
    if tex_data.is_none() {
        panic!("tex_data is null");
    }

    // println!("time:{:?}", time.elapsed());

    let char_size = 64.0;
    let mut world_matrix = Matrix4::<f32>::identity();
    world_matrix =
        world_matrix.append_nonuniform_scaling(&Vector3::<f32>::new(char_size, char_size, 1.0));
    world_matrix = world_matrix.append_translation(&Vector3::<f32>::new(25.0, 120.0, 0.0));

    println!("world_matrix.as_slice(): {:?}", world_matrix.as_slice());
    let world_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(world_matrix.as_slice()),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let view_matrix = Matrix4::<f32>::identity();
    let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(view_matrix.as_slice()),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    println!("view_matrix.as_slice(): {:?}", view_matrix.as_slice());

    let proj_matrix = Orthographic3::<f32>::new(
        0.0,
        window_size.width as f32,
        0.0,
        window_size.height as f32,
        -1.0,
        1.0,
    );

    let proj_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(proj_matrix.as_matrix().as_slice()),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    println!(
        "proj_matrix.as_slice(): {:?}",
        proj_matrix.as_matrix().as_slice()
    );

    let a_glyph_vertex = [
        verties[0].x,
        verties[0].y,
        verties[0].g16hi as f32,
        verties[0].g16lo as f32,
        verties[1].x,
        verties[1].y,
        verties[1].g16hi as f32,
        verties[1].g16lo as f32,
        verties[2].x,
        verties[2].y,
        verties[2].g16hi as f32,
        verties[2].g16lo as f32,
        verties[3].x,
        verties[3].y,
        verties[3].g16hi as f32,
        verties[3].g16lo as f32,
    ];
    println!("a_glyph_vertex: {:?}", a_glyph_vertex);

    let tex = tex_data.as_ref().unwrap();
    let check = tex.cell_size * 0.5 * 2.0f32.sqrt();
    let u_info = [tex.max_offset as f32, tex.min_sdf, tex.sdf_step, check];
    let u_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&u_info),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let font_color: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    let font_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("font_color Buffer"),
        contents: bytemuck::cast_slice(&font_color),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group_layout0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
        ],
    });

    let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout0,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &world_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &view_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &proj_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_info_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &font_color_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
        ],
        label: None,
    });

    let index_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let texture_extent = wgpu::Extent3d {
        width: tex.grid_w as u32,
        height: tex.grid_h as u32,
        depth_or_array_layers: 1,
    };

    let index_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rg8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // println!(
    //     "tex.index_tex: {:?}, tex.grid_w: {}, tex.grid_h:{}, tex.index_tex len: {}",
    //     &tex.index_tex,
    //     tex.grid_w,
    //     tex.grid_h,
    //     tex.index_tex.len()
    // );

    let index_texture_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        index_texture.as_image_copy(),
        &tex.index_tex,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(tex.grid_w as u32 * 2),
            rows_per_image: None,
        },
        texture_extent,
    );

    let bind_group_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout1,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&index_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&index_texture_view),
            },
        ],
        label: None,
    });

    let tex_data_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let texture_extent = wgpu::Extent3d {
        width: tex.data_tex.len() as u32 / 4,
        height: 1,
        depth_or_array_layers: 1,
    };

    let data_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // println!("data_tex: {:?}, len: {}", tex.data_tex, tex.data_tex.len());
    let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        data_texture.as_image_copy(),
        &tex.data_tex,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(tex.data_tex.len() as u32),
            rows_per_image: None,
        },
        texture_extent,
    );

    let bind_group_layout2 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout2,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&tex_data_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&data_texture_view),
            },
        ],
        label: None,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &bind_group_layout0,
            &bind_group_layout1,
            &bind_group_layout2,
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&a_glyph_vertex),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_data = create_indices();
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsages::INDEX,
    });

    let primitive = wgpu::PrimitiveState::default();
    // primitive.
    let mut tt: ColorTargetState = swapchain_format.into();
    tt.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs,
            entry_point: "main",
            targets: &[Some(tt)],
        }),
        primitive,
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // println!("render_pipeline: {:?}", render_pipeline);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        // let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        // println!("=========1");
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                        // timestamp_writes: None,
                        // occlusion_query_set: None,
                    });
                    // rpass.push_debug_group("Prepare data for draw.");
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group0, &[]);
                    rpass.set_bind_group(1, &bind_group1, &[]);
                    rpass.set_bind_group(2, &bind_group2, &[]);
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    // rpass.insert_debug_marker("Draw!");
                    rpass.draw_indexed(0..6, 0, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn create_indices() -> Vec<u16> {
    vec![0, 1, 2, 1, 2, 3]
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .build(&event_loop)
        .unwrap();
    // let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
