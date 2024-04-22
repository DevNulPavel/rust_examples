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
    vec4 u_color;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec3 v_normal;
    float v_alpha;
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_time uniforms.u_time
#define u_color uniforms.u_color

// Varying defines:
#define v_normal input.v_normal
#define v_alpha input.v_alpha
#define v_texcoord input.v_texcoord

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;
    vec3 light = normalize(vec3(-1.0, 0.5, 1.0));
    float coef = max(dot(normalize(v_normal), light), 0.50);
    vec4 color = texture2D(u_texture, v_texcoord);
    color.rgb *= coef * 1.33;
    color.a *= v_alpha;
    output = u_color * color;
    return output;
}
