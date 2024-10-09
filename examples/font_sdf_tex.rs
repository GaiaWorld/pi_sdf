use std::{fmt::UpperHex, mem::transmute, sync::Arc};

use image::ColorType;
use parry2d::na::{self};
use pi_assets::allocator::Allocator;
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

// use nalgebra::Vector3;
use pi_sdf::{blur::gaussian_blur, font::FontFace, glyphy::{blob::TexData, geometry::{aabb::Aabb, arc::Arc as SdfArc}}, utils::{create_indices, CellInfo}};
use pi_wgpu::{self as wgpu, Surface};
use wgpu::{util::DeviceExt, BlendState, ColorTargetState};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let subscriber = Subscriber::builder().with_max_level(Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let window_size = window.inner_size();
    let instance = wgpu::Instance::default();
    // let instance = wgpu::Instance::new(InstanceDescriptor {
    //     backends: Backend::Gl.into(),
    //     dx12_shader_compiler: Dx12Compiler::default(),
    // });

    let surface = instance.create_surface(window.as_ref()).unwrap();
    let surface: Surface<'static> = unsafe { transmute(surface) };
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
    let mut allocator = Allocator::new(128 * 1024 * 1024);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
            &mut allocator
        )
        .await
        .expect("Failed to create device");

    let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../source/sdf.vs").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    // Load the shaders from disk
    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../source/sdf.fs").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    let buffer = std::fs::read("./source/SOURCEHANSANSK-MEDIUM.TTF").unwrap();
    // let buffer = std::fs::read("./source/msyh.ttf").unwrap();
    let mut ft_face = FontFace::new(Arc::new(buffer));
    
    
    println!("max_box_normaliz: {:?}", ft_face.max_box_normaliz());
    let pxrange = 10;
    let time = std::time::Instant::now();
    let mut outline_info = ft_face.to_outline('2');


    // println!("===================plane_bounds: {:?}", plane_bounds);
    let result_arcs = outline_info.compute_near_arcs(2.0);
    // for (indexs, aabb) in &result_arcs.info{
    //     let mut str = "".to_string();
    //     for i in indexs{
    //         str.push_str(&format!("{:?}", result_arcs.arcs[*i]));
    //     }
    //     println!("({:?})", aabb);
    //     println!("")
    // }
    let time2 = std::time::Instant::now();
    // let r = bincode::serialize(&result_arcs).unwrap();
    let r = bitcode::serialize(&result_arcs).unwrap();
    println!("time2: {:?}", (time2.elapsed(), r.len()));
   
    let time2 = std::time::Instant::now();
    // let arcs: CellInfo  = bincode::deserialize(&r).unwrap();
    let arcs: CellInfo  = bitcode::deserialize(&r).unwrap();
    // println!("arcs: {:?}", arcs);
    let weight = 0.0;
    let pxrange = 5;
    let range = 4;
    println!("time3: {:?}", time2.elapsed());
    let time4 = std::time::Instant::now();
    let glpyh_info = outline_info.compute_sdf_tex(arcs, 32, pxrange, false, pxrange);
    // let glpyh_info = FontFace::compute_sdf_tex(outline_info.clone(),  32, pxrange, false);
    println!("time4: {:?}", time4.elapsed());
    // println!("glpyh_info: {:?}", glpyh_info);
    let tex_size = glpyh_info.tex_size;
    let _ = image::save_buffer("image.png", &glpyh_info.sdf_tex, tex_size as u32, tex_size as u32, ColorType::L8);

    let time4 = std::time::Instant::now();
    let gaussian_blur = gaussian_blur(glpyh_info.sdf_tex.clone(), tex_size as u32, tex_size as u32, range, weight);
    println!("time4: {:?}", time4.elapsed());
    let _ = image::save_buffer("gaussian_blur.png", &gaussian_blur, tex_size as u32, tex_size as u32, ColorType::L8);
    // let buffer = include_bytes!("../source/sdf.png").to_vec();
    // let image_buf = image::load_from_memory(&buffer).unwrap();
    // let tex_size: u32 = image_buf.width();
    // let pixmap = image_buf.into_bytes();


    let font_size = 64.0f32;
    let translation = vec![font_size, font_size, 10.0, 10.0];

    let vertexs = [
        0.0f32, 0.0, 0.0, 0.0, 
        0.0, 1.0, 0.0, 1.0, 
        1.0, 0.0, 1.0, 0.0, 
        1.0, 1.0, 1.0, 1.0,
    ]; // 获取网格数据
    println!("vertexs: {:?}", vertexs);

    let view_matrix = na::Matrix4::<f32>::identity(); // 视口矩阵
    let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(view_matrix.as_slice()),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    println!("view_matrix.as_slice(): {:?}", view_matrix.as_slice());

    // 投影矩阵
    let proj_matrix = na::Orthographic3::<f32>::new(
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
    let line = [font_size / 32.0 * 10.0, 0.5, 2.0, 0.0];
    let line_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&line),
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
                    buffer: &view_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &proj_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &line_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
        ],
        label: None,
    });

    // 创建索引纹理
    let sdf_tex = &glpyh_info.sdf_tex;
    let index_texture_extent = wgpu::Extent3d {
        width: tex_size as u32,
        height: tex_size as u32,
        depth_or_array_layers: 1,
    };

    let sdf_tex_sampler = device.create_sampler(&&wgpu::SamplerDescriptor {
        label: None,
        min_filter: wgpu::FilterMode::Linear,
        mag_filter: wgpu::FilterMode::Linear,
        // mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let sdf_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: index_texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let sdf_texture_view = sdf_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        sdf_texture.as_image_copy(),
        sdf_tex,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(tex_size as u32),
            rows_per_image: None,
        },
        index_texture_extent,
    );

    let bind_group_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                resource: wgpu::BindingResource::Sampler(&sdf_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&sdf_texture_view),
            },
        ],
        label: None,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout0, &bind_group_layout1],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[1];
    println!("swapchain_format: {:?}", swapchain_capabilities.formats);
    // 创建网格数据
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&vertexs),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // 以下为实例化数据
    let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("translation_buffer"),
        contents: bytemuck::cast_slice(translation.as_slice()),
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
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 1,
                    }],
                },
            ],
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
        desired_maximum_frame_latency: 2,
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
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    // rpass.push_debug_group("Prepare data for draw.");
                    rpass.set_pipeline(&render_pipeline);

                    rpass.set_bind_group(0, &bind_group0, &[]);
                    rpass.set_bind_group(1, &bind_group1, &[]);

                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, translation_buffer.slice(..));

                    // rpass.insert_debug_marker("Draw!");

                    rpass.draw_indexed(0..6, 0, 0..1 as u32);
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);
    // let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
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
