#version 450
    
precision highp float;


layout(set = 0, binding = 0) uniform mat4 uView;
layout(set = 0, binding = 1) uniform mat4 uProj;


// glyph_vertex_t: x, y; g16hi, g16lo; 
layout (location = 0) in vec4 a_glyph_vertex; // 顶点、uv
layout (location = 1) in vec4 transform; //平移和缩放



layout (location = 1) out vec2 uv;


void main() {
    // vec2 scale = transform.xy;
    // vec2 translation = transform.zw;
    vec2 pos = vec2(a_glyph_vertex.x, a_glyph_vertex.y);
    vec4 pos1 = vec4(pos.x, pos.y, 0.0, 1.0);

    gl_Position = uProj * uView * pos1;

    uv = vec2( a_glyph_vertex.z, 1.0 - a_glyph_vertex.w);
}
