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
    float u_time;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec2 v_texcoord1;
    vec2 v_texcoord2;
    vec3 v_normal;
};

// Attributes defines:
#define a_position input.a_position
#define a_normal input.a_normal
#define a_texcoord input.a_texcoord

// Uniforms defines:
#define u_transform uniforms.u_transform
#define u_time uniforms.u_time

// Varying defines:
#define v_texcoord output.v_texcoord
#define v_texcoord1 output.v_texcoord1
#define v_texcoord2 output.v_texcoord2
#define v_normal output.v_normal
#if defined BUILD_IOS
#define PLATFORM_PRECISION highp
#elif defined BUILD_ANDROID
#define PLATFORM_PRECISION highp
#else
#define PLATFORM_PRECISION
#endif

// Main function
vertex ColorInOut vertFunc(Vertex input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]])
{
    ColorInOut output;
    PLATFORM_PRECISION float xmax = 14173.4;
    PLATFORM_PRECISION float ymax = 9046.35351;

    v_texcoord2 = vec2((a_position.x) / xmax, 1.0 - (a_position.z) / ymax);

    PLATFORM_PRECISION float v_opc = a_position.z / ymax;

    PLATFORM_PRECISION float p = pow((1.0 - v_opc), 2.0);

    v_texcoord = 0.6 * (1.0 + p) * a_texcoord;

    v_texcoord1 = 0.45 * v_texcoord + vec2(12.0, 15.0) * u_time;
    v_texcoord += 6.0 * u_time;

    v_normal = 6.0 * a_normal;
    output.pos = u_transform * a_position;
    return output;
}