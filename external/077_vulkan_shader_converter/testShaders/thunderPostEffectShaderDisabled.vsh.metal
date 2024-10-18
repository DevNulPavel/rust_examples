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

// Attributes:
struct Vertex {
    vec2 a_position [[attribute(0)]];
    vec2 a_texcoord [[attribute(1)]];
};

// Uniforms:
struct ConstantBuffer {
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Attributes defines:
#define a_position input.a_position
#define a_texcoord input.a_texcoord

// Uniforms defines:

// Varying defines:
#define v_texcoord output.v_texcoord

// Main function
vertex ColorInOut vertFunc(Vertex input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]])
{
    ColorInOut output;
    output.pos = vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord;
    return output;
}
