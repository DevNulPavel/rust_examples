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
    float u_height_correction_factor;
    float u_argument_addition;
    float u_angle_k;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec2 v_fragment_screen_coords;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_brightness uniforms.u_brightness
#define u_height_correction_factor uniforms.u_height_correction_factor
#define u_argument_addition uniforms.u_argument_addition
#define u_angle_k uniforms.u_angle_k

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_fragment_screen_coords input.v_fragment_screen_coords

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]])
{
    float4 output;
    PRECISION_LOW vec4 color = texture2D(u_texture, v_texcoord) * u_color * u_brightness;
    PRECISION_HIGH float y = u_angle_k * (v_fragment_screen_coords.x - u_argument_addition);
    PRECISION_HIGH float dy = abs(y - v_fragment_screen_coords.y);

    PRECISION_HIGH float glow_length = 0.2 * u_height_correction_factor;
    PRECISION_HIGH float line_length = 0.02 * u_height_correction_factor;
    PRECISION_HIGH float line_highlight_strength = 0.68;
    PRECISION_HIGH float glow_strength = 0.4;
    PRECISION_HIGH float glow_modifier = clamp((exp( (-dy / glow_length) / 0.5)), 0.0, 1.0);

    color.rgb += color.rgb * (step(dy, line_length) * line_highlight_strength + glow_modifier * glow_strength) * color.a;

    output = color;
    return output;
}

