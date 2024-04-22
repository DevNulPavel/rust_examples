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
    vec2 v_texcoord_caustic1;
    vec2 v_texcoord_caustic2;
};

// Uniforms defines:
#define u_color uniforms.u_color

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_texcoord_caustic1 input.v_texcoord_caustic1
#define v_texcoord_caustic2 input.v_texcoord_caustic2

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], texture2d<float> u_texture2 [[texture(1)]], sampler sampler_u_texture [[sampler(0)]], sampler sampler_u_texture2 [[sampler(1)]])
{
    float4 output;
    vec4 color = texture2D(u_texture, v_texcoord);

    PRECISION_HIGH float s = (color.r + color.g + color.b) * 0.1;
    PRECISION_HIGH vec2 sVec = vec2(s);

    vec4 colorCaustic1 = texture2D(u_texture2, v_texcoord_caustic1 + sVec);
    vec4 colorCaustic2 = texture2D(u_texture2, v_texcoord_caustic2 - sVec);

    vec4 colorCaustic = colorCaustic1 * colorCaustic2 * vec4(1.0, 0.6, 0.2, 1.0);
    colorCaustic.rgb = clamp(colorCaustic.rgb, 0.0, 1.0);

    float y = step(v_texcoord.y, 0.3) * clamp((0.3 - v_texcoord.y) * 10.0, 0.0, 1.0);

    color.rgb += colorCaustic.rgb * y;

    output = clamp(color, 0.0, 1.0);
    return output;
}
