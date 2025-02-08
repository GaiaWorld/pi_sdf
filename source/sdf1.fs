#version 450

#extension GL_OES_standard_derivatives : enable

precision highp float;

#define GLYPHY_INFINITY 1e6
#define GLYPHY_EPSILON  1e-4
#define GLYPHY_MAX_D 0.5
#define GLYPHY_MAX_NUM_ENDPOINTS 20


// (max_offset, min_sdf, sdf_step, check)
// 如果 晶格的 sdf 在 [-check, check]，该晶格 和 字体轮廓 可能 相交 

layout(set = 0, binding = 2) uniform vec4 line; // 渐变

layout(set = 1, binding = 0) uniform sampler sdf_tex_samp;
layout(set = 1, binding = 1) uniform texture2D sdf_tex;
// (网格的边界-宽, 网格的边界-高, z, w)
// z(有效位 低15位) --> (高7位:纹理偏移.x, 中6位:网格宽高.x, 低2位: 00) 
// w(有效位 低15位) --> (高7位:纹理偏移.y, 中6位:网格宽高.y, 低2位: 00) 
layout (location = 1) in vec2 uv; // uv坐标

out vec4 fragColor;
// out float sdf;


void main() {
	float sdf = texture(sampler2D(sdf_tex, sdf_tex_samp), uv).r;
	// fragColor = vec4(fwidth(p), 0., 1.0);

	// 当前到填充边界的像素距离
    float fillSdPx = line.x * (sdf - line.y); 
    float fillOpacity = clamp(fillSdPx + 0.5, 0.0, 1.0);

    float outlineOpacity = 0.0;

    // #ifdef STROKE
    //     // 填充与边框混合
    //     c = mix(strokeColor, c, fillOpacity);

    //     // 当前到描边边界的像素距离
    //     float outlineSdPx = line.x * (sdf - line.z);
    //     outlineOpacity = clamp(outlineSdPx + 0.5, 0.0, 1.0);
    // #endif

    float a = clamp(outlineOpacity + fillOpacity, 0.0, 1.0);
	fragColor = vec4(a, 0.0, 0.0, 1.0);
    // sdf = a;

	// fragColor.rgb *= fragColor.a;
}