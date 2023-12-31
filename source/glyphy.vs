#version 450
    
precision highp float;


layout(set = 0, binding = 0) uniform mat4 uView;
layout(set = 0, binding = 1) uniform mat4 uProj;
// 斜率, 斜体需要
layout(set = 0, binding = 2) uniform vec2 slope;
layout(set = 0, binding = 3) uniform vec2 scale;

// glyph_vertex_t: x, y; g16hi, g16lo; 
layout (location = 0) in vec4 a_glyph_vertex;

layout (location = 1) in vec4 index_info;
layout (location = 2) in vec2 translation;
layout (location = 3) in vec2 data_offset;
layout (location = 4) in vec4 info;

layout (location = 5) out vec2 uv;
layout (location = 6) out vec2 lp;
layout (location = 7) out vec4 u_index_info;
layout (location = 8) out vec2 u_data_offset;
layout (location = 9) out vec4 u_info;



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
}
