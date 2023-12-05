#version 450
    
precision highp float;

layout(set = 0, binding = 0) uniform mat4 uWorld;
layout(set = 0, binding = 1) uniform mat4 uView;
layout(set = 0, binding = 2) uniform mat4 uProj;

// glyph_vertex_t: x, y; g16hi, g16lo; 
layout (location = 0) in vec4 a_glyph_vertex;

out vec4 v_glyph;

// "A" 中 的 v = (26.0, 38.0)
// (26.0, 39.0)
// 27 39
vec4 glyph_vertex_transcode(vec2 v)
{
    // "A", g = (26, 38)
    // (26, 39)
    ivec2 g = ivec2(v);

    // corner = v % 2
    // "A", corner = (0, 0)
    // (0, 1)
    ivec2 corner = ivec2(mod(v, 2.0));

    // "A", g = (13, 19)
    // (13, 19)
    g /= 2;

    // nominal_size = g % 64
    // "A", nominal_size = (13, 19)
    // (13, 19)
    ivec2 nominal_size = ivec2(mod(vec2(g), 64.));

    // "A", return (0.0, 0.0, 52.0, 76.0)
    // (0.0, 19.0, 52.0, 76.0)
    return vec4(corner * nominal_size, g * 4);
}

void main() {
    v_glyph = glyph_vertex_transcode(a_glyph_vertex.zw);

    gl_Position = uProj * uView * uWorld * vec4(a_glyph_vertex.xy, 0.0, 1.0);
}