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
    vec4 a_position [[attribute(0)]];
    vec3 a_normal [[attribute(1)]];
    vec2 a_texcoord [[attribute(2)]];
};

// Uniforms:
struct ConstantBuffer {
    mat4 u_transform;
    float u_flipx;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec4 v_color;
    vec2 v_texcoord;
};

// Attributes defines:
#define a_position input.a_position
#define a_normal input.a_normal
#define a_texcoord input.a_texcoord

// Uniforms defines:
#define u_transform uniforms.u_transform
#define u_flipx uniforms.u_flipx

// Varying defines:
#define v_color output.v_color
#define v_texcoord output.v_texcoord

#if defined BUILD_IOS
#elif defined BUILD_ANDROID
#else
#endif

// Main function
vertex ColorInOut vertFunc(Vertex input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]])
{
    ColorInOut output;
    output.pos = u_transform * vec4(a_position.x * u_flipx, a_position.yzw);
    v_texcoord = a_texcoord;
    return output;
}
