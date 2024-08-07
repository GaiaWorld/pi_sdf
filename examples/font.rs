use std::sync::Arc;

use parry2d::na::{self};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

// use nalgebra::Vector3;
use pi_sdf::{font::FontFace, glyphy::blob::TexData, utils::create_indices};
use pi_wgpu as wgpu;
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

    let surface =  instance.create_surface(window.clone()).unwrap();
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
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
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

    // 加载字体文件
    let buffer = std::fs::read("./source/hwxw.ttf").unwrap();
    // let buffer = std::fs::read("./source/msyh.ttf").unwrap();
    let mut ft_face = FontFace::new(buffer);

    let tex_size = (1024, 1024);
    // 需要渲染的字符串
    let text = "魔".to_string();
    // 纹理数据
    let mut tex_data = TexData {
        index_tex: vec![0; tex_size.0 * tex_size.1 * 2], // 索引纹理数据
        index_offset_x: 0,                               // 从哪个偏移值开始写
        index_offset_y: 0,
        index_tex_width: tex_size.0,                    // 索引纹理宽
        data_tex: vec![0; tex_size.0 * tex_size.1 * 4], // 数据纹理数据
        data_offset_x: 0,
        data_offset_y: 0,
        data_tex_width: tex_size.0,
        sdf_tex: vec![0; tex_size.0 * tex_size.1],
        sdf_tex1: vec![0; tex_size.0 * tex_size.1 / 4],
        sdf_tex2: vec![0; tex_size.0 * tex_size.1 / 16],
        sdf_tex3: vec![0; tex_size.0 * tex_size.1 / 64], // 数据纹理宽
    };
    let time = std::time::Instant::now();
    let texs_info = ft_face.out_tex_data(&text, &mut tex_data).unwrap(); // 将字符串的sdf数据写入纹理
    println!("out_tex_data: {:?}", (time.elapsed(), ));
    
    // 字体缩放
    let scale = [128.0f32, 128.0];

    let translation = vec![
        32.0f32, 32.0, 10.0, 10.0,
        64.,     64.,  80.,  10.,
        128.,     128.,  200.,  10.
    ];

    // 阴影偏移和模糊等级
    let mut shadow_offset_and_blur_level = vec![20.0f32, 20., 0.2, 0.2];
    let vertexs = ft_face.verties(scale[0], &mut shadow_offset_and_blur_level[0..=1]); // 获取网格数据
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

    // 斜体, 第一个值为正切值，第二个写死为网格最小y坐标
    let slope = [0.0, vertexs[1]];
    let slope_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("slope"),
        contents: bytemuck::cast_slice(&slope),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //     label: Some("scale"),
    //     contents: bytemuck::cast_slice(scale.as_slice()),
    //     usage: wgpu::BufferUsages::UNIFORM,
    // });

    let u_gradient_start_end: [f32; 4] = [-0.5, -0.5, 0.5, 0.5];
    let u_gradient_start_end_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("u_gradient_start_end_buffer"),
            contents: bytemuck::cast_slice(&u_gradient_start_end),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let u_weight: [f32; 1] = [1.0];
    let u_weight_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_buffer"),
        contents: bytemuck::cast_slice(&u_weight),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let gradient = [
        1.0f32, 0.0, 0.0, 0.0, // 第一个
        1.0f32, 0.0, 0.0, 0.4, // 第二个
        0.0f32, 0.0, 1.0, 0.6, // 第三个
        1.0f32, 1.0, 0.0, 1.0, // 第四个
    ];
    let u_gradient_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("gradient"),
        contents: bytemuck::cast_slice(&gradient),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let outer_glow_color_and_dist = vec![1.0f32, 0.84, 0.0, 5.0];
    let outer_glow_color_and_dist_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("outer_glow_color_and_dist"),
            contents: bytemuck::cast_slice(&outer_glow_color_and_dist),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let shadow_color = vec![0.0f32, 0.0, 0.0, 1.0];
    let shadow_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("shadow_color"),
        contents: bytemuck::cast_slice(&shadow_color),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let shadow_offset_and_blur_level_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow_offset_and_blur_level"),
            contents: bytemuck::cast_slice(&shadow_offset_and_blur_level),
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
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4),
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
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 8,
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
                    buffer: &slope_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(8),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_weight_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(4),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_gradient_start_end_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &u_gradient_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &outer_glow_color_and_dist_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_color_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_offset_and_blur_level_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            },
        ],
        label: None,
    });

    // 创建索引纹理
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
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

    let sdf_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_weight_and_offset_buffer"),
        contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let sdf_tex = [
        &tex_data.sdf_tex,
        &tex_data.sdf_tex1,
        &tex_data.sdf_tex2,
        &tex_data.sdf_tex3,
    ];
    let mut sdf_texture_extent = wgpu::Extent3d {
        width: tex_size.0 as u32,
        height: tex_size.1 as u32,
        depth_or_array_layers: 1,
    };
    let sdf_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        min_filter: wgpu::FilterMode::Linear,
        mag_filter: wgpu::FilterMode::Linear,
        // mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let mip_level_count = 4;
    let sdf_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: sdf_texture_extent,
        mip_level_count: mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let sdf_texture_view = sdf_texture.create_view(&wgpu::TextureViewDescriptor::default());
    for i in 0..=(mip_level_count - 1) {
        sdf_texture_extent = wgpu::Extent3d {
            width: tex_size.0 as u32 >> i,
            height: tex_size.1 as u32 >> i,
            depth_or_array_layers: 1,
        };
        println!("sdf{}: {}", i, sdf_tex[i as usize][0]);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &sdf_texture,
                mip_level: i,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            sdf_tex[i as usize],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(tex_size.0 as u32 >> i),
                rows_per_image: None,
            },
            sdf_texture_extent,
        );
    }

    let bind_group_layout3 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                resource: wgpu::BindingResource::Sampler(&sdf_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&sdf_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &sdf_tex_size_buffer,
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
    let swapchain_format = swapchain_capabilities.formats[1];
    println!("swapchain_format: {:?}", swapchain_capabilities.formats);
    // 创建网格数据
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&vertexs),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // 以下为实例化数据
   
    let mut index_info = vec![];
    let mut data_offset = vec![];
    let mut u_info = vec![];
    let mut fill_color = vec![0.0f32; texs_info.len() * 4];
    let mut stroke_color_and_width = vec![0.0f32; texs_info.len() * 4];
    let mut start_and_step = Vec::with_capacity(texs_info.len() * 4);

    let mut index = 0;

    for info in &texs_info {
        index_info.push(info.index_offset_x as f32); // 每个字符索引纹理的偏移
        index_info.push(info.index_offset_y as f32);
        index_info.push(info.grid_w); // 每个字符索引纹理宽
        index_info.push(info.grid_w); // 每个字符索引纹理高

        data_offset.push(info.data_offset_x as f32); // 每个字符数据纹理的偏移
        data_offset.push(info.data_offset_y as f32); // 每个字符数据纹理的偏移

        // sdf 附带的信息
        let check = info.cell_size * 0.5 * 2.0f32.sqrt();
        u_info.push(info.max_offset as f32);
        u_info.push(info.min_sdf as f32);
        u_info.push(info.sdf_step as f32);
        u_info.push(check);

        // 每个字符的位置
        let x = index % 15;
        let y = index / 15;
       
        fill_color[index * 4] = 1.0;
        fill_color[index * 4 + 1] = 0.0;
        fill_color[index * 4 + 2] = 0.0;
        fill_color[index * 4 + 3] = 1.0;

        stroke_color_and_width[index * 4] = 0.0;
        stroke_color_and_width[index * 4 + 1] = 1.0;
        stroke_color_and_width[index * 4 + 2] = 0.0;
        stroke_color_and_width[index * 4 + 3] = 0.0;

        start_and_step.push(0.0f32);
        start_and_step.push(0.0);
        start_and_step.push(10000000.0);
        start_and_step.push(10000000.0);

        index += 1;
    }

    println!("index_info: {:?}", index_info);
    println!("translation: {:?}", translation);
    println!("data_offset: {:?}", data_offset);
    println!("u_info: {:?}", u_info);
    println!("fill_color: {:?}", fill_color);
    println!("stroke_color_and_width: {:?}", stroke_color_and_width);
    println!("start_and_step: {:?}", start_and_step);

    let index_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index_info_buffer"),
        contents: bytemuck::cast_slice(index_info.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("translation_buffer"),
        contents: bytemuck::cast_slice(translation.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let data_offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("data_offset_buffer"),
        contents: bytemuck::cast_slice(data_offset.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let u_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("u_info_buffer"),
        contents: bytemuck::cast_slice(&u_info),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let fill_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("font_color Buffer"),
        contents: bytemuck::cast_slice(&fill_color),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let stroke_color_and_width_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("stroke_color_and_width_buffer"),
            contents: bytemuck::cast_slice(&stroke_color_and_width),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let start_and_step_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("start_and_step_buffer"),
        contents: bytemuck::cast_slice(&start_and_step),
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
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
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
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 7,
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

    let len = text.chars().count();
    println!("len = {},text: {}", len, text);

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
                    rpass.set_vertex_buffer(7, start_and_step_buffer.slice(..));
                    // rpass.insert_debug_marker("Draw!");

                    rpass.draw_indexed(0..6, 0, 0..len as u32);
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
