// use std::sync::Arc;

// use parry2d::na::{self};
// use tracing::Level;
// use tracing_subscriber::fmt::Subscriber;

// // use nalgebra::Vector3;
// use pi_sdf::{glyphy::blob::TexData, svg::Svg, utils::create_indices};
// use pi_wgpu as wgpu;
// use wgpu::{util::DeviceExt, BlendState, ColorTargetState};
// use winit::{
//     event::{Event, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     window::{Window, WindowBuilder},
// };

// async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
//     let subscriber = Subscriber::builder().with_max_level(Level::TRACE).finish();

//     tracing::subscriber::set_global_default(subscriber).unwrap();

//     let window_size = window.inner_size();
//     let instance = wgpu::Instance::default();
//     // let instance = wgpu::Instance::new(InstanceDescriptor {
//     //     backends: Backend::Gl.into(),
//     //     dx12_shader_compiler: Dx12Compiler::default(),
//     // });

//     let surface = { instance.create_surface(window.clone()) }.unwrap();
//     let adapter = instance
//         .request_adapter(&wgpu::RequestAdapterOptions {
//             power_preference: wgpu::PowerPreference::default(),
//             force_fallback_adapter: false,
//             // Request an adapter which can render to our surface
//             compatible_surface: Some(&surface),
//         })
//         .await
//         .expect("Failed to find an appropriate adapter");

//     // Create the logical device and command queue
//     let (device, queue) = adapter
//         .request_device(
//             &wgpu::DeviceDescriptor {
//                 label: None,
//                 required_features: wgpu::Features::empty(),
//                 // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
//                 required_limits: wgpu::Limits::downlevel_webgl2_defaults()
//                     .using_resolution(adapter.limits()),
//             },
//             None,
//         )
//         .await
//         .expect("Failed to create device");

//     let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: None,
//         source: wgpu::ShaderSource::Glsl {
//             shader: include_str!("../source/glyphy.vs").into(),
//             stage: naga::ShaderStage::Vertex,
//             defines: Default::default(),
//         },
//     });

//     // Load the shaders from disk
//     let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//         label: None,
//         source: wgpu::ShaderSource::Glsl {
//             shader: include_str!("../source/glyphy.fs").into(),
//             stage: naga::ShaderStage::Fragment,
//             defines: Default::default(),
//         },
//     });

//     // log::debug!("vs: {:?}", vs);
//     // log::debug!("fs: {:?}", fs);
//     let buffer = std::fs::read("svg.svg").unwrap();
//     let mut svg = Svg::new(buffer);

//     // let time = std::time::Instant::now();
//     let tex_size = (1024, 1024);

//     let mut tex_data = TexData {
//         index_tex: vec![0; tex_size.0 * tex_size.1 * 2],
//         index_offset_x: 0,
//         index_offset_y: 0,
//         index_tex_width: tex_size.0,
//         data_tex: vec![0; tex_size.0 * tex_size.1 * 4],
//         data_offset_x: 0,
//         data_offset_y: 0,
//         data_tex_width: tex_size.0,
//         sdf_tex: vec![0; tex_size.0 * tex_size.1],
//         sdf_tex1: vec![0; tex_size.0 * tex_size.1 / 4],
//         sdf_tex2: vec![0; tex_size.0 * tex_size.1 / 16],
//         sdf_tex3: vec![0; tex_size.0 * tex_size.1 / 64], // 数据纹理宽
//     };
//     let time = std::time::Instant::now();
//     let (texs_info, attributes) = svg.out_tex_data(&mut tex_data).unwrap();
//     log::debug!("out_tex_data: {:?}", time.elapsed());
//     let vertexs = svg.verties();
//     // 字体缩放
//     let scale = [1.0f32, 1.0];
//     // 阴影偏移和模糊等级
//     let mut shadow_offset_and_blur_level = vec![20.0f32, 0., 6.0, 0.0];
//     log::debug!("vertexs: {:?}", vertexs);

//     let view_matrix = na::Matrix4::<f32>::identity(); // 视口矩阵
//     let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("Index Buffer"),
//         contents: bytemuck::cast_slice(view_matrix.as_slice()),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });
//     log::debug!("view_matrix.as_slice(): {:?}", view_matrix.as_slice());

//     let proj_matrix = na::Orthographic3::<f32>::new(
//         0.0,
//         window_size.width as f32,
//         0.0,
//         window_size.height as f32,
//         -1.0,
//         1.0,
//     );
//     let proj_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("Index Buffer"),
//         contents: bytemuck::cast_slice(proj_matrix.as_matrix().as_slice()),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });
//     log::debug!(
//         "proj_matrix.as_slice(): {:?}",
//         proj_matrix.as_matrix().as_slice()
//     );

//     // 斜体, 第一个值为正切值，第二个写死为网格最小y坐标
//     let slope = [0.0, vertexs[1]];
//     let slope_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("slope"),
//         contents: bytemuck::cast_slice(&slope),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("scale"),
//         contents: bytemuck::cast_slice(scale.as_slice()),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let u_gradient_start_end: [f32; 4] = [-0.5, -0.5, 0.5, 0.5];
//     let u_gradient_start_end_buffer =
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("u_gradient_start_end_buffer"),
//             contents: bytemuck::cast_slice(&u_gradient_start_end),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//     let u_weight: [f32; 1] = [0.0];
//     let u_weight_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("u_weight_buffer"),
//         contents: bytemuck::cast_slice(&u_weight),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let gradient = [
//         1.0f32, 0.0, 0.0, 0.0, // 第一个
//         1.0f32, 0.0, 0.0, 0.4, // 第二个
//         0.0f32, 0.0, 1.0, 0.6, // 第三个
//         1.0f32, 1.0, 0.0, 1.0, // 第四个
//     ];
//     let u_gradient_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("gradient"),
//         contents: bytemuck::cast_slice(&gradient),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let outer_glow_color_and_dist = vec![0.0f32, 0.0, 0.0, 0.0];
//     let outer_glow_color_and_dist_buffer =
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("outer_glow_color_and_dist"),
//             contents: bytemuck::cast_slice(&outer_glow_color_and_dist),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//     let shadow_color = vec![0.0f32, 0.0, 0.0, 0.0];
//     let shadow_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("shadow_color"),
//         contents: bytemuck::cast_slice(&shadow_color),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let shadow_offset_and_blur_level_buffer =
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("shadow_offset_and_blur_level"),
//             contents: bytemuck::cast_slice(&shadow_offset_and_blur_level),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//     let bind_group_layout0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: None,
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(64),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(64),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(8),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 3,
//                 visibility: wgpu::ShaderStages::VERTEX,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(8),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 4,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(4),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 5,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(16),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 6,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(64),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 7,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(16),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 8,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(16),
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 9,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(16),
//                 },
//                 count: None,
//             },
//         ],
//     });

//     let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: &bind_group_layout0,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &view_matrix_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(64),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &proj_matrix_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(64),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &slope_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(8),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 3,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &scale_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(8),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 4,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &u_weight_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(4),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 5,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &u_gradient_start_end_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(16),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 6,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &u_gradient_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(64),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 7,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &outer_glow_color_and_dist_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(16),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 8,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &shadow_color_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(16),
//                 }),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 9,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &shadow_offset_and_blur_level_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(16),
//                 }),
//             },
//         ],
//         label: None,
//     });

//     // 创建索引纹理
//     let index_tex = &tex_data.index_tex;
//     let index_texture_extent = wgpu::Extent3d {
//         width: tex_size.0 as u32,
//         height: tex_size.1 as u32,
//         depth_or_array_layers: 1,
//     };
//     let index_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("u_weight_and_offset_buffer"),
//         contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });
//     let index_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
//     let index_texture = device.create_texture(&wgpu::TextureDescriptor {
//         label: None,
//         size: index_texture_extent,
//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Rg8Unorm,
//         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//         view_formats: &[],
//     });
//     let index_texture_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());
//     queue.write_texture(
//         index_texture.as_image_copy(),
//         index_tex,
//         wgpu::ImageDataLayout {
//             offset: 0,
//             bytes_per_row: Some(tex_size.0 as u32 * 2),
//             rows_per_image: None,
//         },
//         index_texture_extent,
//     );

//     let bind_group_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: None,
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Texture {
//                     multisampled: false,
//                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     view_dimension: wgpu::TextureViewDimension::D2,
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(8),
//                 },
//                 count: None,
//             },
//         ],
//     });

//     let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: &bind_group_layout1,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Sampler(&index_tex_sampler),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::TextureView(&index_texture_view),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &index_tex_size_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(8),
//                 }),
//             },
//         ],
//         label: None,
//     });

//     let data_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("u_weight_and_offset_buffer"),
//         contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });

//     let data_tex = &tex_data.data_tex;
//     let data_texture_extent = wgpu::Extent3d {
//         width: tex_size.0 as u32,
//         height: tex_size.1 as u32,
//         depth_or_array_layers: 1,
//     };
//     let data_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
//     let data_texture = device.create_texture(&wgpu::TextureDescriptor {
//         label: None,
//         size: data_texture_extent,
//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Rgba8Unorm,
//         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//         view_formats: &[],
//     });
//     let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
//     queue.write_texture(
//         data_texture.as_image_copy(),
//         data_tex,
//         wgpu::ImageDataLayout {
//             offset: 0,
//             bytes_per_row: Some(tex_size.0 as u32 * 4),
//             rows_per_image: None,
//         },
//         data_texture_extent,
//     );

//     let bind_group_layout2 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: None,
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Texture {
//                     multisampled: false,
//                     sample_type: wgpu::TextureSampleType::Float { filterable: false },
//                     view_dimension: wgpu::TextureViewDimension::D2,
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(8),
//                 },
//                 count: None,
//             },
//         ],
//     });
//     let bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: &bind_group_layout2,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Sampler(&data_tex_sampler),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::TextureView(&data_texture_view),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &data_tex_size_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(8),
//                 }),
//             },
//         ],
//         label: None,
//     });

//     let sdf_tex_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("u_weight_and_offset_buffer"),
//         contents: bytemuck::cast_slice(&[tex_size.0 as f32, tex_size.1 as f32]),
//         usage: wgpu::BufferUsages::UNIFORM,
//     });
//     let sdf_tex = [
//         &tex_data.sdf_tex,
//         &tex_data.sdf_tex1,
//         &tex_data.sdf_tex2,
//         &tex_data.sdf_tex3,
//     ];
//     let mut sdf_texture_extent = wgpu::Extent3d {
//         width: tex_size.0 as u32,
//         height: tex_size.1 as u32,
//         depth_or_array_layers: 1,
//     };
//     let sdf_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//         label: None,
//         min_filter: wgpu::FilterMode::Linear,
//         mag_filter: wgpu::FilterMode::Linear,
//         // mipmap_filter: wgpu::FilterMode::Linear,
//         ..Default::default()
//     });
//     let mip_level_count = 4;
//     let sdf_texture = device.create_texture(&wgpu::TextureDescriptor {
//         label: None,
//         size: sdf_texture_extent,
//         mip_level_count: mip_level_count,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::R8Unorm,
//         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//         view_formats: &[],
//     });

//     let sdf_texture_view = sdf_texture.create_view(&wgpu::TextureViewDescriptor::default());
//     for i in 0..=(mip_level_count - 1) {
//         sdf_texture_extent = wgpu::Extent3d {
//             width: tex_size.0 as u32 >> i,
//             height: tex_size.1 as u32 >> i,
//             depth_or_array_layers: 1,
//         };
//         log::debug!("sdf{}: {}", i, sdf_tex[i as usize][0]);
//         queue.write_texture(
//             wgpu::ImageCopyTexture {
//                 texture: &sdf_texture,
//                 mip_level: i,
//                 origin: wgpu::Origin3d::ZERO,
//                 aspect: wgpu::TextureAspect::All,
//             },
//             sdf_tex[i as usize],
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(tex_size.0 as u32 >> i),
//                 rows_per_image: None,
//             },
//             sdf_texture_extent,
//         );
//     }

//     let bind_group_layout3 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: None,
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Texture {
//                     multisampled: false,
//                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     view_dimension: wgpu::TextureViewDimension::D2,
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Buffer {
//                     ty: wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: wgpu::BufferSize::new(8),
//                 },
//                 count: None,
//             },
//         ],
//     });

//     let bind_group3 = device.create_bind_group(&wgpu::BindGroupDescriptor {
//         layout: &bind_group_layout3,
//         entries: &[
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Sampler(&sdf_tex_sampler),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 1,
//                 resource: wgpu::BindingResource::TextureView(&sdf_texture_view),
//             },
//             wgpu::BindGroupEntry {
//                 binding: 2,
//                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                     buffer: &sdf_tex_size_buffer,
//                     offset: 0,
//                     size: wgpu::BufferSize::new(8),
//                 }),
//             },
//         ],
//         label: None,
//     });

//     let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: None,
//         bind_group_layouts: &[
//             &bind_group_layout0,
//             &bind_group_layout1,
//             &bind_group_layout2,
//             &bind_group_layout3,
//         ],
//         push_constant_ranges: &[],
//     });

//     let swapchain_capabilities = surface.get_capabilities(&adapter);
//     let swapchain_format = swapchain_capabilities.formats[1];
//     log::debug!("swapchain_format: {:?}", swapchain_capabilities.formats);
//     // 创建网格数据
//     let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("Index Buffer"),
//         contents: bytemuck::cast_slice(&vertexs),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     // svg中有多少个标签就有多少个以下数据
//     let mut translation = vec![]; // 每个标签的位置
//     let mut index_info = vec![]; // 每个标签的索引纹理的偏移和宽高
//     let mut data_offset = vec![]; // 每个标签的数据纹偏移
//     let mut u_info = vec![]; // 每个标签的sdf信息
//     let mut fill_color = vec![0.0; attributes.len() * 4]; // 每个标签的填充颜色
//     let mut stroke_color_and_width = vec![0.0; attributes.len() * 4]; // 每个标签的描边颜色和描边宽度
//     let mut start_and_step = Vec::with_capacity(attributes.len() * 4); // 每个标签的虚线描边信息
//     for info in &texs_info {
//         translation.push(0.0);
//         translation.push(0.0);

//         index_info.push(info.index_offset.0 as f32);
//         index_info.push(info.index_offset.1 as f32);
//         index_info.push(info.grid_w);
//         index_info.push(info.grid_h);

//         data_offset.push(info.data_offset.0 as f32);
//         data_offset.push(info.data_offset.1 as f32);
//         let check = info.cell_size * 0.5 * 2.0f32.sqrt();
//         u_info.push(info.max_offset as f32);
//         u_info.push(info.min_sdf as f32);
//         u_info.push(info.sdf_step as f32);
//         u_info.push(check);
//     }
//     log::debug!("index_info: {:?}", index_info);
//     log::debug!("translation: {:?}", translation);
//     log::debug!("data_offset: {:?}", data_offset);
//     log::debug!("u_info: {:?}", u_info);

//     let mut index = 0;
//     for attr in &attributes {
//         if let Some(fill) = &attr.fill {
//             if let usvg::Paint::Color(color) = fill.paint {
//                 fill_color[index * 4] = color.red as f32 / 255.0;
//                 fill_color[index * 4 + 1] = color.green as f32 / 255.0;
//                 fill_color[index * 4 + 2] = color.blue as f32 / 255.0;
//                 // 如果不是封闭路径，将填充颜色设置为0
//                 fill_color[index * 4 + 3] = if attr.is_close { 1.0 } else { 0.0 };
//             }
//         }

//         // 当不需要虚线时，见这两个值填无穷大
//         let mut step = [100000.0, 100000.0];
//         if let Some(stroke) = &attr.stroke {
//             if let usvg::Paint::Color(color) = stroke.paint {
//                 stroke_color_and_width[index * 4] = color.red as f32 / 255.0;
//                 stroke_color_and_width[index * 4 + 1] = color.green as f32 / 255.0;
//                 stroke_color_and_width[index * 4 + 2] = color.blue as f32 / 255.0;
//                 // 如果不需要描边时，将描边宽度设置为0.0
//                 stroke_color_and_width[index * 4 + 3] = stroke.width.get();
//             }

//             if let Some(dasharray) = &stroke.dasharray {
//                 step[0] = dasharray[0];
//                 step[1] = dasharray[1];
//             }
//         }

//         start_and_step.push(attr.start.x);
//         start_and_step.push(attr.start.y);
//         start_and_step.push(step[0]);
//         start_and_step.push(step[1]);

//         index += 1;
//     }
//     log::debug!("fill_color: {:?}", fill_color);
//     log::debug!("stroke_color_and_width: {:?}", stroke_color_and_width);
//     log::debug!("start_and_step: {:?}", start_and_step);

//     let index_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("index_info_buffer"),
//         contents: bytemuck::cast_slice(index_info.as_slice()),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("translation_buffer"),
//         contents: bytemuck::cast_slice(translation.as_slice()),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let data_offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("data_offset_buffer"),
//         contents: bytemuck::cast_slice(data_offset.as_slice()),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let u_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("u_info_buffer"),
//         contents: bytemuck::cast_slice(&u_info),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let fill_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("font_color Buffer"),
//         contents: bytemuck::cast_slice(&fill_color),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let stroke_color_and_width_buffer =
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("stroke_color_and_width_buffer"),
//             contents: bytemuck::cast_slice(&stroke_color_and_width),
//             usage: wgpu::BufferUsages::VERTEX,
//         });

//     let start_and_step_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("start_and_step_buffer"),
//         contents: bytemuck::cast_slice(&start_and_step),
//         usage: wgpu::BufferUsages::VERTEX,
//     });

//     let index_data = create_indices();
//     let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("Index Buffer"),
//         contents: bytemuck::cast_slice(&index_data),
//         usage: wgpu::BufferUsages::INDEX,
//     });

//     let primitive = wgpu::PrimitiveState::default();

//     // primitive.
//     let mut tt: ColorTargetState = swapchain_format.into();
//     tt.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
//     let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: None,
//         layout: Some(&pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: &vs,
//             entry_point: "main",
//             buffers: &[
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Vertex,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 0,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 1,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x2,
//                         offset: 0,
//                         shader_location: 2,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x2,
//                         offset: 0,
//                         shader_location: 3,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 4,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 5,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 6,
//                     }],
//                 },
//                 wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Instance,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 7,
//                     }],
//                 },
//             ],
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &fs,
//             entry_point: "main",
//             targets: &[Some(tt)],
//         }),
//         primitive,
//         depth_stencil: None,
//         multisample: wgpu::MultisampleState::default(),
//         multiview: None,
//     });

//     // log::debug!("render_pipeline: {:?}", render_pipeline);

//     let mut config = wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format: swapchain_format,
//         width: window_size.width,
//         height: window_size.height,
//         present_mode: wgpu::PresentMode::Fifo,
//         alpha_mode: swapchain_capabilities.alpha_modes[0],
//         view_formats: vec![],
//         desired_maximum_frame_latency: 2,
//     };

//     surface.configure(&device, &config);
//     let count = texs_info.len();

//     event_loop.run(move |event, _, control_flow| {
//         // Have the closure take ownership of the resources.
//         // `event_loop.run` never returns, therefore we must do this to ensure
//         // the resources are properly cleaned up.
//         // let _ = (&instance, &adapter, &shader, &pipeline_layout);

//         *control_flow = ControlFlow::Wait;
//         // log::debug!("=========1");
//         match event {
//             Event::WindowEvent {
//                 event: WindowEvent::Resized(size),
//                 ..
//             } => {
//                 // Reconfigure the surface with the new size
//                 config.width = size.width;
//                 config.height = size.height;
//                 surface.configure(&device, &config);
//                 // On macos the window needs to be redrawn manually after resizing
//                 window.request_redraw();
//             }
//             Event::MainEventsCleared => {
//                 window.request_redraw();
//             }
//             Event::RedrawRequested(_) => {
//                 let frame = surface
//                     .get_current_texture()
//                     .expect("Failed to acquire next swap chain texture");
//                 let view = frame
//                     .texture
//                     .create_view(&wgpu::TextureViewDescriptor::default());
//                 let mut encoder =
//                     device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//                 {
//                     let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                         label: None,
//                         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                             view: &view,
//                             resolve_target: None,
//                             ops: wgpu::Operations {
//                                 load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
//                                 store: wgpu::StoreOp::Store,
//                             },
//                         })],
//                         depth_stencil_attachment: None,
//                         timestamp_writes: None,
//                         occlusion_query_set: None,
//                     });
//                     // rpass.push_debug_group("Prepare data for draw.");
//                     rpass.set_pipeline(&render_pipeline);

//                     rpass.set_bind_group(0, &bind_group0, &[]);
//                     rpass.set_bind_group(1, &bind_group1, &[]);
//                     rpass.set_bind_group(2, &bind_group2, &[]);
//                     rpass.set_bind_group(3, &bind_group3, &[]);

//                     rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

//                     rpass.set_vertex_buffer(0, vertex_buffer.slice(..));

//                     rpass.set_vertex_buffer(1, index_info_buffer.slice(..));
//                     rpass.set_vertex_buffer(2, translation_buffer.slice(..));
//                     rpass.set_vertex_buffer(3, data_offset_buffer.slice(..));
//                     rpass.set_vertex_buffer(4, u_info_buffer.slice(..));
//                     rpass.set_vertex_buffer(5, fill_color_buffer.slice(..));
//                     rpass.set_vertex_buffer(6, stroke_color_and_width_buffer.slice(..));
//                     rpass.set_vertex_buffer(7, start_and_step_buffer.slice(..));
//                     // rpass.insert_debug_marker("Draw!");

//                     rpass.draw_indexed(0..6, 0, 0..count as u32);
//                 }

//                 queue.submit(Some(encoder.finish()));
//                 frame.present();
//             }
//             Event::WindowEvent {
//                 event: WindowEvent::CloseRequested,
//                 ..
//             } => *control_flow = ControlFlow::Exit,
//             _ => {}
//         }
//     });
// }

fn main() {
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new()
//         .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
//         .build(&event_loop)
//         .unwrap();
//     let window = Arc::new(window);
//     #[cfg(not(target_arch = "wasm32"))]
//     {
//         env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
//         pollster::block_on(run(event_loop, window));
//     }
//     #[cfg(target_arch = "wasm32")]
//     {
//         std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//         console_log::init().expect("could not initialize logger");
//         use winit::platform::web::WindowExtWebSys;
//         // On wasm, append the canvas to the document body
//         web_sys::window()
//             .and_then(|win| win.document())
//             .and_then(|doc| doc.body())
//             .and_then(|body| {
//                 body.append_child(&web_sys::Element::from(window.canvas()))
//                     .ok()
//             })
//             .expect("couldn't append canvas to document body");
//         wasm_bindgen_futures::spawn_local(run(event_loop, window));
//     }
}
