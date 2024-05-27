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
    float u_speed;
    float u_tex_scaling;
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
#define u_speed uniforms.u_speed
#define u_tex_scaling uniforms.u_tex_scaling

// Varying defines:
#define v_texcoord input.v_texcoord
#if defined BUILD_IOS
            #define FLOAT highp float
    #define VEC2 highp vec2
#elif defined BUILD_ANDROID
            #define FLOAT highp float
    #define VEC2 highp vec2
#else
            #define FLOAT float
    #define VEC2 vec2
#endif

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;
    vec4 color = u_color * texture2D(u_texture, v_texcoord);

    float tex_coord = v_texcoord.y / u_tex_scaling;

    vec4 result_color;
    result_color.w = u_color.w;
    result_color.xyz = mix(color.xyz, vec3(0.3490196, 0.8980392, 0.9372549) * tex_coord, tex_coord);
    output = clamp(result_color, 0.0, 1.0);
    return output;
}