#version 450
    
precision highp float;


layout(set = 0, binding = 0) uniform mat4 uView;
layout(set = 0, binding = 1) uniform mat4 uProj;
// 斜率, 斜体需要
layout(set = 0, binding = 2) uniform vec2 slope; // 正切值，原点y值
layout(set = 0, binding = 3) uniform vec2 scale;

// glyph_vertex_t: x, y; g16hi, g16lo; 
layout (location = 0) in vec4 a_glyph_vertex; // 顶点、uv

layout (location = 1) in vec4 index_info; // 索引纹理宽高（晶格个数）， 索引纹理偏移（单位： 像素）
layout (location = 2) in vec2 translation; // 位置
layout (location = 3) in vec2 data_offset; // 数据纹理偏移
layout (location = 4) in vec4 info; // sdf信息;
layout (location = 5) in vec4 fillColor; 
layout (location = 6) in vec4 strokeColorAndWidth;
layout (location = 7) in vec4 startAndStep;



layout (location = 1) out vec2 uv;
layout (location = 2) out vec2 lp;
layout (location = 3) out vec4 u_index_info;
layout (location = 4) out vec2 u_data_offset;
layout (location = 5) out vec4 u_info;
layout (location = 6) out vec4 u_fillColor;
layout (location = 7) out vec4 u_strokeColorAndWidth;
layout (location = 8) out vec2 u_pos;
layout (location = 9) out vec4 u_startAndStep;

void main() {
    float x = (slope.y - a_glyph_vertex.y) * slope.x;
    vec2 pos = vec2(a_glyph_vertex.x - x, a_glyph_vertex.y);
    vec4 pos1 = vec4(pos.x * scale.x + translation.x, pos.y * scale.y + translation.y, 0.0, 1.0);

    gl_Position = uProj * uView * pos1;

    lp = vec2(mix(-0.5, 0.5, step(0., a_glyph_vertex.x)), mix(-0.5, 0.5, step(0., a_glyph_vertex.y)));
    uv = a_glyph_vertex.zw;
    u_index_info = index_info;
    u_data_offset = data_offset;
    u_info = info;
    u_fillColor = fillColor;
    u_strokeColorAndWidth = strokeColorAndWidth;
    u_pos = vec2(pos1.x, pos1.y);
    u_startAndStep = startAndStep;
}
