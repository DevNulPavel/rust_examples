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
    vec4 v_blurTexCoords_0;
    vec4 v_blurTexCoords_1;
    vec4 v_blurTexCoords_2;
    vec4 v_blurTexCoords_3;
    vec4 v_blurTexCoords_4;
    vec4 v_blurTexCoords_5;
    vec4 v_blurTexCoords_6;
};

// Uniforms defines:

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_blurTexCoords_0 input.v_blurTexCoords_0
#define v_blurTexCoords_1 input.v_blurTexCoords_1
#define v_blurTexCoords_2 input.v_blurTexCoords_2
#define v_blurTexCoords_3 input.v_blurTexCoords_3
#define v_blurTexCoords_4 input.v_blurTexCoords_4
#define v_blurTexCoords_5 input.v_blurTexCoords_5
#define v_blurTexCoords_6 input.v_blurTexCoords_6

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;

    vec4 color = texture2D(u_texture, v_texcoord)*0.159576912161;

    color += texture2D(u_texture, v_blurTexCoords_0.xy)*0.0044299121055113265;
    color += texture2D(u_texture, v_blurTexCoords_0.zw)*0.00895781211794;
    color += texture2D(u_texture, v_blurTexCoords_1.xy)*0.0215963866053;
    color += texture2D(u_texture, v_blurTexCoords_1.zw)*0.0443683338718;
    color += texture2D(u_texture, v_blurTexCoords_2.xy)*0.0776744219933;
    color += texture2D(u_texture, v_blurTexCoords_2.zw)*0.115876621105;
    color += texture2D(u_texture, v_blurTexCoords_3.xy)*0.147308056121;

    color += texture2D(u_texture, v_blurTexCoords_3.zw)*0.147308056121;
    color += texture2D(u_texture, v_blurTexCoords_4.xy)*0.115876621105;
    color += texture2D(u_texture, v_blurTexCoords_4.zw)*0.0776744219933;
    color += texture2D(u_texture, v_blurTexCoords_5.xy)*0.0443683338718;
    color += texture2D(u_texture, v_blurTexCoords_5.zw)*0.0215963866053;
    color += texture2D(u_texture, v_blurTexCoords_6.xy)*0.00895781211794;
    color += texture2D(u_texture, v_blurTexCoords_6.zw)*0.0044299121055113265;

    output = color;
    return output;
}