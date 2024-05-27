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
    float u_angle;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_angle uniforms.u_angle

// Varying defines:
#define v_texcoord input.v_texcoord

#define M_PI 3.1415926535897932384626433832795

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]]) {
    float4 output;
    vec4 color = texture2D(u_texture, v_texcoord);   
    vec2 delta = v_texcoord - vec2(0.5);

    float ang = - atan( -delta.x, delta.y );
    ang = ang - M_PI * 2.0 * floor(ang / (M_PI * 2.0) );

    output = u_color * color * vec4(float(ang < u_angle));
    return output;
}
