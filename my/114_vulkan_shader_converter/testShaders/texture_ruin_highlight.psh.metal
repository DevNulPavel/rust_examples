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
    vec3 v_texcoord;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_brightness uniforms.u_brightness
#define u_time uniforms.u_time

// Varying defines:
#define v_texcoord input.v_texcoord

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;
    vec4 color = u_color * texture2D(u_texture, v_texcoord.xy);
    color *= clamp(max(v_texcoord.z - 20.0, 0.0)*0.010, 0.0, 1.0);
    color.rgb = clamp(color.rgb * u_brightness + vec3(0.15, 0.17, 0.16) * color.a, 0.0, 1.0);

    output = color;
    return output;
}
