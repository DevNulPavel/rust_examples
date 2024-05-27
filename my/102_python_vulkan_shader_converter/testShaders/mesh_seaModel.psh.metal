#include <metal_stdlib>
#include <simd/simd.h>

#define PRECISION_HIGH
#define PRECISION_MEDIUM
#define PRECISION_LOW

#define vec2 float2
#define vec3 float3
#define vec4 float4
#define mat3x4 float3x4
#define mat3 float3x3
#define mat4 float4x4
#define ivec int
#define uvec uint
#define bvec bool
#define atan(x,y) atan2(x,y)

#define texture2D(TEXTURE_NAME, UV_COORD) TEXTURE_NAME.sample(sampler_##TEXTURE_NAME, UV_COORD)

using namespace metal;

// Uniforms:
struct ConstantBuffer {
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec2 v_texcoord1;
    vec2 v_texcoord2;
    vec3 v_normal;
};

// Uniforms defines:

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_texcoord1 input.v_texcoord1
#define v_texcoord2 input.v_texcoord2
#define v_normal input.v_normal
#if defined BUILD_IOS
#define PLATFORM_PRECISION highp
#elif defined BUILD_ANDROID
#define PLATFORM_PRECISION highp
#else
#define PLATFORM_PRECISION
#endif

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], texture2d<float> u_texture2 [[texture(1)]], sampler sampler_u_texture [[sampler(0)]], sampler sampler_u_texture2 [[sampler(1)]])
{
    float4 output;
        vec4 waterColor1 = texture2D(u_texture, v_texcoord);
        vec4 waterColor2 = texture2D(u_texture, v_texcoord1);
        vec4 color2 = texture2D(u_texture2, v_texcoord2);

        vec4 waterColor = (waterColor1 + waterColor2);
        float foam = clamp(1.25 * (color2.g - 0.2), 0.0, 1.0);
        float opacity = color2.r - foam * 0.2;
        float light = color2.b;

        vec4 colorNew = mix(vec4(0.0, 0.5, 0.8, opacity), waterColor1, 0.25);
        vec4 color = mix(vec4(0.35, 0.9, 0.55, opacity), colorNew, opacity);

        float wSum = (waterColor.r + waterColor.g + waterColor.b);

        float wFoam = wSum * (wSum - 3.2);

        float border = clamp(v_normal.b, 0.0, 1.0);
        wFoam = (clamp(wFoam * 0.275 + foam * 2.0 * border, 0.0, 1.0));

        color = mix(color, vec4(1.0, 1.0, 1.0, 1.0), wFoam);         color = mix(color, vec4(0.376, 0.894, 0.901, 1.0), light);

        color.a = mix(color.a, 0.0, border);
        output = color;
    return output;
}
