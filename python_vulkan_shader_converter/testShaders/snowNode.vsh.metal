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
    vec3 a_position [[attribute(0)]];
    float a_opacity [[attribute(1)]];
    vec2 a_texCoord [[attribute(2)]];
};

// Uniforms:
struct ConstantBuffer {
    mat4 u_transformCustom;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    float v_opacity;
    vec2 v_texCoord;
};

// Attributes defines:
#define a_position input.a_position
#define a_opacity input.a_opacity
#define a_texCoord input.a_texCoord

// Uniforms defines:
#define u_transformCustom uniforms.u_transformCustom

// Varying defines:
#define v_opacity output.v_opacity
#define v_texCoord output.v_texCoord

// Main function
vertex ColorInOut vertFunc(Vertex input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]])
{
    ColorInOut output;
    output.pos = u_transformCustom * vec4(a_position, 1.0);
    v_opacity = a_opacity;
    v_texCoord = a_texCoord;
    return output;
}
