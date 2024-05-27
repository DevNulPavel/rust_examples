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
    vec2 a_texcoord [[attribute(1)]];
    vec4 a_color [[attribute(2)]];
};

// Uniforms:
struct ConstantBuffer {
    mat4 u_transform;
    float u_time2;
    vec4 u_color;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec4 v_color;
    vec4 v_darken;
};

// Attributes defines:
#define a_position input.a_position
#define a_texcoord input.a_texcoord
#define a_color input.a_color

// Uniforms defines:
#define u_transform uniforms.u_transform
#define u_time2 uniforms.u_time2
#define u_color uniforms.u_color

// Varying defines:
#define v_texcoord output.v_texcoord
#define v_color output.v_color
#define v_darken output.v_darken
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
    PLATFORM_PRECISION float param = u_time2 + (a_position.x + a_position.y) * 0.0025;
    PLATFORM_PRECISION float sin_p = sin(param);
    PLATFORM_PRECISION float cos_p = cos(param);
    float trigonometryMult = 0.05;
    v_texcoord = vec2(a_texcoord.x + sin_p * trigonometryMult, a_texcoord.y + cos_p * trigonometryMult);
    v_color = a_color;
    v_darken = mix(vec4(0.0, 0.0, 0.0, 0.0), u_color, v_color.a);
    output.pos = u_transform * a_position;
    return output;
}
