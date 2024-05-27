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
    float u_mtime;
    vec4 u_center;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_mtime uniforms.u_mtime
#define u_center uniforms.u_center

// Varying defines:
#define v_texcoord input.v_texcoord

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]]) {
    float4 output;
    vec2 pos = gl_FragCoord.xy;

    vec2 center = u_center.xy;

    vec2 dif = pos - center;
    float distance = length(dif);

    float t = u_mtime;
    float time = t * 1000 + t * t * 0.5f * -2500.0f;

    vec3 shockParams = vec3(10.0, 0.8, 50);
    vec2 texCoord = v_texcoord;

        float diff = (distance - time); 
        float powDiff = exp(-(diff*diff)/100); 
        float diffTime = powDiff * 0.01; 
        vec2 diffUV = normalize(dif); 
        texCoord = v_texcoord + (diffUV * diffTime);

    vec4 color = texture2D(u_texture, texCoord);
    output = u_color * color;
    return output;
}
