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
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_color uniforms.u_color

// Varying defines:
#define v_texcoord input.v_texcoord

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;
    vec4 color = u_color * texture2D(u_texture, v_texcoord);
    const vec3 grayColor = vec3(0.299, 0.587, 0.114);
    float gray = dot(mix(color.rgb, vec3(1.0), 0.4), grayColor) * 1.05;

    const vec3 sepia = vec3(1.1290, 1.0081, 0.8103);

    output = vec4(vec3(gray * sepia.r, gray * sepia.g, gray * sepia.b)*color.a, color.a);
    return output;
}
