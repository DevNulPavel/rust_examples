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
};

// Uniforms:
struct ConstantBuffer {
    mat4 u_transform;
    float hPxSize;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec4 v_blurTexCoords_0;
    vec4 v_blurTexCoords_1;
    vec4 v_blurTexCoords_2;
    vec4 v_blurTexCoords_3;
    vec4 v_blurTexCoords_4;
    vec4 v_blurTexCoords_5;
    vec4 v_blurTexCoords_6;
};

// Attributes defines:
#define a_position input.a_position
#define a_texcoord input.a_texcoord

// Uniforms defines:
#define u_transform uniforms.u_transform
#define hPxSize uniforms.hPxSize

// Varying defines:
#define v_texcoord output.v_texcoord
#define v_blurTexCoords_0 output.v_blurTexCoords_0
#define v_blurTexCoords_1 output.v_blurTexCoords_1
#define v_blurTexCoords_2 output.v_blurTexCoords_2
#define v_blurTexCoords_3 output.v_blurTexCoords_3
#define v_blurTexCoords_4 output.v_blurTexCoords_4
#define v_blurTexCoords_5 output.v_blurTexCoords_5
#define v_blurTexCoords_6 output.v_blurTexCoords_6

#if defined BUILD_IOS
#elif defined BUILD_ANDROID
#else
#endif

// Main function
vertex ColorInOut vertFunc(Vertex input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]])
{
    ColorInOut output;
    output.pos = u_transform * a_position;
    v_texcoord = a_texcoord;

    v_blurTexCoords_0.xy = v_texcoord + vec2(-hPxSize * 7.0, 0.0);
    v_blurTexCoords_0.zw = v_texcoord + vec2(-hPxSize * 6.0, 0.0);
    v_blurTexCoords_1.xy = v_texcoord + vec2(-hPxSize * 5.0, 0.0);
    v_blurTexCoords_1.zw = v_texcoord + vec2(-hPxSize * 4.0, 0.0);
    v_blurTexCoords_2.xy = v_texcoord + vec2(-hPxSize * 3.0, 0.0);
    v_blurTexCoords_2.zw = v_texcoord + vec2(-hPxSize * 2.0, 0.0);
    v_blurTexCoords_3.xy = v_texcoord + vec2(-hPxSize, 0.0);
    v_blurTexCoords_3.zw = v_texcoord + vec2( hPxSize, 0.0);
    v_blurTexCoords_4.xy = v_texcoord + vec2( hPxSize * 2.0, 0.0);
    v_blurTexCoords_4.zw = v_texcoord + vec2( hPxSize * 3.0, 0.0);
    v_blurTexCoords_5.xy = v_texcoord + vec2( hPxSize * 4.0, 0.0);
    v_blurTexCoords_5.zw = v_texcoord + vec2( hPxSize * 5.0, 0.0);
    v_blurTexCoords_6.xy = v_texcoord + vec2( hPxSize * 6.0, 0.0);
    v_blurTexCoords_6.zw = v_texcoord + vec2( hPxSize * 7.0, 0.0);

    return output;
}