// use crate::{
//     glyphy::{
//         blob_new::TexData,
//         vertex::{add_glyph_vertices, GlyphInfo, GlyphyVertex},
//     },
//     glyphy_draw2::get_char_arc,
//     utils::{FontFace, GlyphVisitor},
//     Matrix4, Orthographic3, Point, Vector3,
// };
// use pi_wgpu as wgpu;
// use wgpu::util::DeviceExt;

// #[derive(Debug, Default, Clone, Copy, PartialEq)]
// pub struct Color {
//     r: f32,
//     g: f32,
//     b: f32,
//     a: f32,
// }

// #[derive(Debug, Default, PartialEq)]
// pub struct ColorGradient {
//     start_gradient: Point,
//     end_gradient: Point,
//     color_step: Vec<(f32, Color)>,
// }

// impl ColorGradient {
//     pub fn new(start: Point, end: Point) -> Self {
//         Self {
//             start_gradient: start,
//             end_gradient: end,
//             color_step: Vec::with_capacity(4),
//         }
//     }

//     pub fn add_color_stop(&mut self, step: f32, color: Color) {
//         self.color_step.push((step, color))
//     }
// }

// pub enum OutlineType {
//     Font,
//     Svg,
// }

// pub struct RenderPath {
//     proj_matrix: Orthographic3,
//     view_matrix: Matrix4,

//     render_update: bool,
//     path_update: bool,
//     fill_color: Color,
//     stroke_color: Color,
//     stroke_width: f32,
//     slope: f32,
//     color_gradient: ColorGradient,
//     font_size: u32,
//     outline_type: OutlineType,
//     outline: Option<GlyphVisitor>,
//     pos: Point,

//     font_face: Option<FontFace>,

//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     render_pipeline: wgpu::RenderPipeline,
//     bind_group_layout0: wgpu::BindGroupLayout,
//     bind_group_layout1: wgpu::BindGroupLayout,
//     bind_group_layout2: wgpu::BindGroupLayout,
// }

// impl RenderPath {
//     pub fn new(
//         device: wgpu::Device,
//         queue: wgpu::Queue,
//         window_width: f32,
//         window_height: f32,
//     ) -> Self {
//         let proj_matrix = Orthographic3::new(0.0, window_width, 0.0, window_height, -1.0, 1.0);

//         let view_matrix = Matrix4::identity();

//         let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: None,
//             source: wgpu::ShaderSource::Glsl {
//                 shader: include_str!("../source/glyphy.vs").into(),
//                 stage: naga::ShaderStage::Vertex,
//                 defines: Default::default(),
//             },
//         });

//         // Load the shaders from disk
//         let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: None,
//             source: wgpu::ShaderSource::Glsl {
//                 shader: include_str!("../source/glyphy.fs").into(),
//                 stage: naga::ShaderStage::Fragment,
//                 defines: Default::default(),
//             },
//         });

//         let bind_group_layout0 = Self::create_bind_group_layout0(&device);
//         let bind_group_layout1 = Self::create_bind_group_layout1(&device);
//         let bind_group_layout2 = Self::create_bind_group_layout2(&device);

//         let render_pipeline = Self::create_render_pipeline(
//             &device,
//             &vs,
//             &fs,
//             &bind_group_layout0,
//             &bind_group_layout1,
//             &bind_group_layout2,
//         );

//         RenderPath {
//             proj_matrix,
//             view_matrix,

//             render_update: true,
//             path_update: true,
//             stroke_width: 0.0,
//             fill_color: Color::default(),
//             stroke_color: Color::default(),
//             slope: 0.0,
//             color_gradient: ColorGradient::default(),
//             font_size: 20,
//             outline: None,
//             outline_type: OutlineType::Font,
//             pos: Point::default(),

//             font_face: None,

//             device,
//             queue,
//             render_pipeline,
//             bind_group_layout0,
//             bind_group_layout1,
//             bind_group_layout2,
//         }
//     }

//     pub fn set_stroke_width(&mut self, w: f32) {
//         let u_outline: [f32; 4] = [0.2, 0.9, 0.2, 2.0];
//         let u_outline_buffer = self
//             .device
//             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some("u_outline_buffer"),
//                 contents: bytemuck::cast_slice(&u_outline),
//                 usage: wgpu::BufferUsages::UNIFORM,
//             });
//         if w != self.stroke_width {
//             self.stroke_width = w;
//             self.render_update = true;
//         }
//     }

//     pub fn set_fill_color(&mut self, color: Color) {
//         if self.fill_color != color {
//             self.fill_color = color;
//             self.render_update = true;
//         }
//     }

//     pub fn set_stroke_color(&mut self, color: Color) {
//         if self.stroke_color != color {
//             self.stroke_color = color;
//             self.render_update = true;
//         }
//     }

//     pub fn set_slope(&mut self, slope: f32) {
//         if self.slope != slope {
//             self.slope = slope;
//             self.render_update = true;
//         }
//     }

//     pub fn set_fill_gradient(&mut self, color_gradient: ColorGradient) {
//         if self.color_gradient != color_gradient {
//             self.color_gradient = color_gradient;
//             self.render_update = true;
//         }
//     }

//     pub fn set_font_size(&mut self, size: u32) {
//         if self.font_size != size {
//             self.font_size = size;
//             self.render_update = true;
//         }
//     }

//     pub fn set_pos(&mut self, pos: Point) {
//         if self.pos != pos {
//             self.pos = pos;
//             self.render_update = true;
//         }
//     }

//     pub fn set_font_buffer(&mut self, buffer: Vec<u8>) {
//         let face = FontFace::new(buffer);
//         self.font_face = Some(face);
//     }

//     pub fn draw_arc(&mut self, verties: &[GlyphyVertex; 4], ) {
//         let device = &self.device;
//         let mut gi = GlyphInfo::new();
//         let arcs = get_char_arc(&mut gi, self.font_face.as_mut().unwrap(), char, None);

//         let verties = add_glyph_vertices(&gi, None, None);

//         let a_glyph_vertex = [
//             verties[0].x,
//             verties[0].y,
//             verties[0].g16hi as f32,
//             verties[0].g16lo as f32,
//             verties[1].x,
//             verties[1].y,
//             verties[1].g16hi as f32,
//             verties[1].g16lo as f32,
//             verties[2].x,
//             verties[2].y,
//             verties[2].g16hi as f32,
//             verties[2].g16lo as f32,
//             verties[3].x,
//             verties[3].y,
//             verties[3].g16hi as f32,
//             verties[3].g16lo as f32,
//         ];
//         log::debug!("a_glyph_vertex: {:?}", a_glyph_vertex);
//         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(&a_glyph_vertex),
//             usage: wgpu::BufferUsages::VERTEX,
//         });

//         let index_data = create_indices();
//         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(&index_data),
//             usage: wgpu::BufferUsages::INDEX,
//         });

//         let tex_data = arcs.tex_data.as_ref().unwrap();
//         let bind_group0 = self.create_bind_group0(&verties, tex_data, pos);
//         let bind_group1 = self.create_bind_group1(tex_data);
//         let bind_group2 = self.create_bind_group2(tex_data);
//     }

//     fn create_bind_group_layout0(device: &wgpu::Device) -> wgpu::BindGroupLayout {
//         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("bind_group_layout0"),
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(64),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(64),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(64),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 3,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(8),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 4,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(16),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 5,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(16),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 6,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(16),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 7,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(16),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 8,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(16),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 9,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer {
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: wgpu::BufferSize::new(64),
//                     },
//                     count: None,
//                 },
//             ],
//         })
//     }

//     fn create_bind_group_layout1(device: &wgpu::Device) -> wgpu::BindGroupLayout {
//         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("index_tex"),
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         multisampled: false,
//                         sample_type: wgpu::TextureSampleType::Float { filterable: false },
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                     },
//                     count: None,
//                 },
//             ],
//         })
//     }

//     fn create_bind_group_layout2(device: &wgpu::Device) -> wgpu::BindGroupLayout {
//         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("data_tex"),
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Texture {
//                         multisampled: false,
//                         sample_type: wgpu::TextureSampleType::Float { filterable: false },
//                         view_dimension: wgpu::TextureViewDimension::D2,
//                     },
//                     count: None,
//                 },
//             ],
//         })
//     }

//     fn create_bind_group0(
//         &self,
//         verties: &[GlyphyVertex; 4],
//         tex_data: &TexData,
//         pos: Point,
//     ) -> wgpu::BindGroup {
//         let device = &self.device;
//         let world_matrix = Matrix4::identity()
//             .append_nonuniform_scaling(&Vector3::new(
//                 self.font_size as f32,
//                 self.font_size as f32,
//                 1.0,
//             ))
//             .append_translation(&Vector3::new(pos.x, pos.y, 0.0));

//         log::debug!("world_matrix.as_slice(): {:?}", world_matrix.as_slice());
//         let world_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(world_matrix.as_slice()),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(self.view_matrix.as_slice()),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });
//         log::debug!("view_matrix.as_slice(): {:?}", self.view_matrix.as_slice());

//         let proj_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(self.proj_matrix.as_matrix().as_slice()),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         log::debug!(
//             "proj_matrix.as_slice(): {:?}",
//             self.proj_matrix.as_matrix().as_slice()
//         );

//         let slope = [self.slope, verties[0].y];
//         let slope_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(&slope),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         let check = tex_data.cell_size * 0.5 * 2.0f32.sqrt();
//         let u_info = [
//             tex_data.max_offset as f32,
//             tex_data.min_sdf,
//             tex_data.sdf_step,
//             check,
//         ];
//         let u_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("u_info"),
//             contents: bytemuck::cast_slice(&u_info),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });
//         let font_color: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
//         let font_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("font_color Buffer"),
//             contents: bytemuck::cast_slice(&font_color),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         let u_gradient_start_end = [
//             self.color_gradient.start_gradient.x,
//             self.color_gradient.start_gradient.y,
//             self.color_gradient.end_gradient.x,
//             self.color_gradient.end_gradient.y,
//         ];
//         let u_gradient_start_end_buffer =
//             device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some("u_gradient_start_end_buffer"),
//                 contents: bytemuck::cast_slice(&u_gradient_start_end),
//                 usage: wgpu::BufferUsages::UNIFORM,
//             });

//         let u_outline: [f32; 4] = [0.2, 0.9, 0.2, 2.0];
//         let u_outline_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("u_outline_buffer"),
//             contents: bytemuck::cast_slice(&u_outline),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         let u_weight_and_offset: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
//         let u_weight_and_offset_buffer =
//             device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some("u_weight_and_offset_buffer"),
//                 contents: bytemuck::cast_slice(&u_weight_and_offset),
//                 usage: wgpu::BufferUsages::UNIFORM,
//             });

//         let mut gradient = Vec::with_capacity(16);
//         if let Some(gradien) = self.color_gradient.color_step.get(0) {
//             gradient.extend([gradien.0, gradien.1.r, gradien.1.g, gradien.1.b])
//         } else {
//             gradient.extend([0., 0., 0., 0.0]);
//         };

//         if let Some(gradien) = self.color_gradient.color_step.get(1) {
//             gradient.extend([gradien.0, gradien.1.r, gradien.1.g, gradien.1.b])
//         } else {
//             gradient.extend([0., 0., 0., 0.0]);
//         };

//         if let Some(gradien) = self.color_gradient.color_step.get(2) {
//             gradient.extend([gradien.0, gradien.1.r, gradien.1.g, gradien.1.b])
//         } else {
//             gradient.extend([0., 0., 0., 0.0]);
//         };

//         if let Some(gradien) = self.color_gradient.color_step.get(3) {
//             gradient.extend([gradien.0, gradien.1.r, gradien.1.g, gradien.1.b])
//         } else {
//             gradient.extend([0., 0., 0., 0.0]);
//         };
//         let gradient = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("u_weight_and_offset_buffer"),
//             contents: bytemuck::cast_slice(&gradient),
//             usage: wgpu::BufferUsages::UNIFORM,
//         });

//         device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &self.bind_group_layout0,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &world_matrix_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(64),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &view_matrix_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(64),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &proj_matrix_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(64),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 3,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &slope_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(8),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 4,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &u_info_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(16),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 5,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &font_color_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(16),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 6,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &u_gradient_start_end_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(16),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 7,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &u_outline_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(16),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 8,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &u_weight_and_offset_buffer,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(16),
//                     }),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 9,
//                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                         buffer: &gradient,
//                         offset: 0,
//                         size: wgpu::BufferSize::new(64),
//                     }),
//                 },
//             ],
//             label: None,
//         })
//     }

//     fn create_bind_group1(&self, tex_data: &TexData) -> wgpu::BindGroup {
//         let device = &self.device;
//         let queue = &self.queue;

//         let index_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

//         let texture_extent = wgpu::Extent3d {
//             width: tex_data.grid_w as u32,
//             height: tex_data.grid_h as u32,
//             depth_or_array_layers: 1,
//         };

//         let index_texture = device.create_texture(&wgpu::TextureDescriptor {
//             label: None,
//             size: texture_extent,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rg8Unorm,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         // log::debug!(
//         //     "tex.index_tex: {:?}, tex.grid_w: {}, tex.grid_h:{}, tex.index_tex len: {}",
//         //     &tex.index_tex,
//         //     tex.grid_w,
//         //     tex.grid_h,
//         //     tex.index_tex.len()
//         // );

//         let index_texture_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());
//         queue.write_texture(
//             index_texture.as_image_copy(),
//             &tex_data.index_tex,
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(tex_data.grid_w as u32 * 2),
//                 rows_per_image: None,
//             },
//             texture_extent,
//         );

//         device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &self.bind_group_layout1,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::Sampler(&index_tex_sampler),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::TextureView(&index_texture_view),
//                 },
//             ],
//             label: None,
//         })
//     }

//     fn create_bind_group2(&self, tex_data: &TexData) -> wgpu::BindGroup {
//         let device = &self.device;
//         let queue = &self.queue;

//         let tex_data_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
//         let texture_extent = wgpu::Extent3d {
//             width: tex_data.data_tex.len() as u32 / 4,
//             height: 1,
//             depth_or_array_layers: 1,
//         };

//         let data_texture = device.create_texture(&wgpu::TextureDescriptor {
//             label: None,
//             size: texture_extent,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8Unorm,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         // log::debug!("data_tex: {:?}, len: {}", tex.data_tex, tex.data_tex.len());
//         let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
//         queue.write_texture(
//             data_texture.as_image_copy(),
//             &tex_data.data_tex,
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: Some(tex_data.data_tex.len() as u32),
//                 rows_per_image: None,
//             },
//             texture_extent,
//         );

//         device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: &self.bind_group_layout2,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::Sampler(&tex_data_sampler),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::TextureView(&data_texture_view),
//                 },
//             ],
//             label: None,
//         })
//     }

//     fn create_render_pipeline(
//         device: &wgpu::Device,
//         vs: &wgpu::ShaderModule,
//         fs: &wgpu::ShaderModule,
//         layout0: &wgpu::BindGroupLayout,
//         layout1: &wgpu::BindGroupLayout,
//         layout2: &wgpu::BindGroupLayout,
//     ) -> wgpu::RenderPipeline {
//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: None,
//             bind_group_layouts: &[layout0, layout1, layout2],
//             push_constant_ranges: &[],
//         });

//         let primitive = wgpu::PrimitiveState::default();

//         let mut tt: wgpu::ColorTargetState = wgpu::TextureFormat::Rgba8UnormSrgb.into();
//         tt.blend = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);

//         device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: None,
//             layout: Some(&pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: vs,
//                 entry_point: "main",
//                 buffers: &[wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     step_mode: wgpu::VertexStepMode::Vertex,
//                     attributes: &[wgpu::VertexAttribute {
//                         format: wgpu::VertexFormat::Float32x4,
//                         offset: 0,
//                         shader_location: 0,
//                     }],
//                 }],
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: fs,
//                 entry_point: "main",
//                 targets: &[Some(tt)],
//             }),
//             primitive,
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview: None,
//         })
//     }
// }

// fn create_indices() -> Vec<u16> {
//     vec![0, 1, 2, 1, 2, 3]
// }
