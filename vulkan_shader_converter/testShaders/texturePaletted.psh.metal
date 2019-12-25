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
    vec4 u_color;
    float u_brightness;
    float u_time;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_brightness uniforms.u_brightness
#define u_time uniforms.u_time

// Varying defines:
#define v_texcoord input.v_texcoord

#if defined BUILD_IOS
#else
#endif

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], texture2d<float> u_texture [[texture(1)]], texture2d<float> u_texture8 [[texture(2)]], sampler sampler_u_texture [[sampler(0)]], sampler sampler_u_texture [[sampler(1)]], sampler sampler_u_texture8 [[sampler(2)]])
{
    float4 output;
    vec2 index = vec2(texture2D(u_texture, v_texcoord).r, 0.0);
    vec4 color = texture2D(u_texture8, index);
    color.rgb = u_color.rgb * clamp(color.rgb * u_brightness, 0.0, 1.0);
    output = color;
    return output;
}
