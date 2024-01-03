use env_logger::Env;
use parry2d::na::{self, Vector3};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

// use nalgebra::Vector3;
use pi_sdf::{font::FontFace, glyphy::blob::TexData, svg::Svg2, utils::create_indices};
use pi_wgpu as wgpu;
use usvg::Color;
use wgpu::{
    util::DeviceExt, Backend, BlendState, ColorTargetState, Dx12Compiler, InstanceDescriptor,
};
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
    // let instance = wgpu::Instance::new(InstanceDescriptor {
    //     backends: Backend::Gl.into(),
    //     dx12_shader_compiler: Dx12Compiler::default(),
    // });

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
            shader: include_str!("../source/svg.vs").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    // Load the shaders from disk
    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../source/svg.fs").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    // println!("vs: {:?}", vs);
    // println!("fs: {:?}", fs);
    let buffer = std::fs::read("svg.svg").unwrap();
    let mut svg = Svg2::new(buffer);

    // let time = std::time::Instant::now();
    let tex_size = (1024, 1024);

    let mut tex_data = TexData {
        index_tex: vec![0; tex_size.0 * tex_size.1 * 2],
        index_offset_x: 0,
        index_offset_y: 0,
        index_tex_width: tex_size.0,
        data_tex: vec![0; tex_size.0 * tex_size.1 * 4],
        data_offset_x: 0,
        data_offset_y: 0,
        data_tex_width: tex_size.0,
    };
    let time = std::time::Instant::now();
    let (texs_info, attributes) = svg.out_tex_data(&mut tex_data).unwrap();
    println!("out_tex_data: {:?}", time.elapsed());
    let vertexs = svg.verties();
    println!("vertexs: {:?}", vertexs);

    let view_matrix = na::Matrix4::<f32>::identity();
    let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(view_matrix.as_slice()),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    println!("view_matrix.as_slice(): {:?}", view_matrix.as_slice());

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

    let slope = [0.0, vertexs[1]];
    let slope_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&slope),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let scale = [1.0f32, 1.0];
    let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(scale.as_slice()),
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
                    min_binding_size: wgpu::BufferSize::new(8),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(8),
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
                    buffer: &slope_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(8),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scale_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(8),
                }),
            },
        ],
        label: None,
    });

    let u_gradient_start_end: [f32; 4] = [-0.5, -0.5, 0.5, 0.5];
    let u_gradient_start_end_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("u_gradient_start_end_buffer"),
            contents: bytemuck::cast_slice(&u_gradient_start_end),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let u_weight_and_offset: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
    let u_weight_and_offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_and_offset_buffer"),
        contents: bytemuck::cast_slice(&u_weight_and_offset),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let gradient = [
        1.0f32, 0.0, 0.0, 0.0, // 第一个
        1.0f32, 0.0, 0.0, 0.4, // 第二个
        0.0f32, 0.0, 1.0, 0.6, // 第三个
        1.0f32, 1.0, 0.0, 1.0, // 第四个
    ];
    let u_gradient_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_and_offset_buffer"),
        contents: bytemuck::cast_slice(&gradient),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
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
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_weight_and_offset_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_gradient_start_end_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_gradient_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
        ],
        label: None,
    });

    let index_tex = &tex_data.index_tex;
    let index_texture_extent = wgpu::Extent3d {
        width: tex_size.0 as u32,
        height: tex_size.1 as u32,
        depth_or_array_layers: 1,
    };
    let index_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_and_offset_buffer"),
        contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let index_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let index_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: index_texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rg8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let index_texture_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        index_texture.as_image_copy(),
        index_tex,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(tex_size.0 as u32 * 2),
            rows_per_image: None,
        },
        index_texture_extent,
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(8),
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
                resource: wgpu::BindingResource::Sampler(&index_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&index_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &index_tex_size_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(8),
                }),
            },
        ],
        label: None,
    });

    let data_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_and_offset_buffer"),
        contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let data_tex = &tex_data.data_tex;
    let data_texture_extent = wgpu::Extent3d {
        width: tex_size.0 as u32,
        height: tex_size.1 as u32,
        depth_or_array_layers: 1,
    };
    let data_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let data_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: data_texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        data_texture.as_image_copy(),
        data_tex,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(tex_size.0 as u32 * 4),
            rows_per_image: None,
        },
        data_texture_extent,
    );

    let bind_group_layout3 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(8),
                },
                count: None,
            },
        ],
    });

    let bind_group3 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout3,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&data_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&data_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &data_tex_size_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(8),
                }),
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
            &bind_group_layout3,
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&vertexs),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let translation = [
        0.0f32, 0.0, // 第一个字位置
        0.0, 0.0, // 第二个字位置
        0.0, 0.0, // 第一个字位置
        0.0, 0.0, // 第二个字位置
        0.0, 0.0, // 第一个字位置
        0.0, 0.0, // 第二个字位置
        0.0, 0.0, // 第一个字位置
        0.0, 0.0, // 第二个字位置
    ];
    let mut index_info = vec![];
    let mut data_offset = vec![];
    let mut u_info = vec![];
    let mut fill_color = vec![0.0; attributes.len() * 4];
    let mut stroke_color_and_width = vec![0.0; attributes.len() * 4];

    for info in &texs_info {
        index_info.push(info.index_offset.0 as f32);
        index_info.push(info.index_offset.1 as f32);
        index_info.push(32.0);
        index_info.push(32.0);

        data_offset.push(info.data_offset.0 as f32);
        data_offset.push(info.data_offset.1 as f32);

        let check = info.cell_size * 0.5 * 2.0f32.sqrt();
        u_info.push(info.max_offset as f32);
        u_info.push(info.min_sdf as f32);
        u_info.push(info.sdf_step as f32);
        u_info.push(check);
    }
    println!("index_info: {:?}", index_info);
    println!("translation: {:?}", translation);
    println!("data_offset: {:?}", data_offset);
    println!("u_info: {:?}", u_info);

    let mut index = 0;
    for attr in &attributes {
        let fill = attr.fill.as_ref().unwrap();

        if let usvg::Paint::Color(color) = fill.paint {
            fill_color[index * 4] = color.red as f32 / 255.0;
            fill_color[index * 4 + 1] = color.green as f32 / 255.0;
            fill_color[index * 4 + 2] = color.blue as f32 / 255.0;
            fill_color[index * 4 + 3] = 1.0;
        } else {
            fill_color[index * 4 + 3] = 1.0;
        }

        if let Some(stroke) = &attr.stroke {
            if let usvg::Paint::Color(color) = stroke.paint {
                stroke_color_and_width[index * 4] = color.red as f32 / 255.0;
                stroke_color_and_width[index * 4 + 1] = color.green as f32 / 255.0;
                stroke_color_and_width[index * 4 + 2] = color.blue as f32 / 255.0;
                stroke_color_and_width[index * 4 + 3] = stroke.width.get() * 3.0;
            }
        }
        index += 1;
    }
    println!("fill_color: {:?}", fill_color);
    println!("stroke_color_and_width: {:?}", stroke_color_and_width);

    let index_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(index_info.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(translation.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let data_offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(data_offset.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let u_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&u_info),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let fill_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("font_color Buffer"),
        contents: bytemuck::cast_slice(&fill_color),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let stroke_color_and_width_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_outline_buffer"),
        contents: bytemuck::cast_slice(&stroke_color_and_width),
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
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 2,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 3,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 4,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 5,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 6,
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
    };

    surface.configure(&device, &config);
    let count = texs_info.len();

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
                    rpass.set_bind_group(3, &bind_group3, &[]);

                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    rpass.set_vertex_buffer(1, index_info_buffer.slice(..));
                    rpass.set_vertex_buffer(2, translation_buffer.slice(..));
                    rpass.set_vertex_buffer(3, data_offset_buffer.slice(..));
                    rpass.set_vertex_buffer(4, u_info_buffer.slice(..));
                    rpass.set_vertex_buffer(5, fill_color_buffer.slice(..));
                    rpass.set_vertex_buffer(6, stroke_color_and_width_buffer.slice(..));
                    // rpass.insert_debug_marker("Draw!");

                    rpass.draw_indexed(0..6, 0, 0..count as u32);
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
