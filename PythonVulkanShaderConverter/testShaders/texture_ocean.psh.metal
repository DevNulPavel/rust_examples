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
    float u_time;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    float v_timesin;
};

// Uniforms defines:
#define u_time uniforms.u_time

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_timesin input.v_timesin

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], texture2d<float> u_texture2 [[texture(1)]], sampler sampler_u_texture [[sampler(0)]], sampler sampler_u_texture2 [[sampler(1)]])
{
    float4 output;
    PRECISION_HIGH vec4 flow = texture2D(u_texture2, v_texcoord);
    PRECISION_HIGH float str = 1.0 / (flow.w * 255.0) * v_timesin / 3.0;
    flow.xy = (flow.xy * 2.0 - 1.0) * str;

    output = texture2D(u_texture, v_texcoord + flow.xy);
    return output;
}
