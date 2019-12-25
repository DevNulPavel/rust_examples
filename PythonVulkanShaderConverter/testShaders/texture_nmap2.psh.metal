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
    float u_time2;
    vec4 world_pos;
    float pixels_x;
    float pixels_y;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
    vec4 v_worldpos;
    vec4 v_position;
};

// Uniforms defines:
#define u_color uniforms.u_color
#define u_time2 uniforms.u_time2
#define world_pos uniforms.world_pos
#define pixels_x uniforms.pixels_x
#define pixels_y uniforms.pixels_y

// Varying defines:
#define v_texcoord input.v_texcoord
#define v_worldpos input.v_worldpos
#define v_position input.v_position

#if defined BUILD_IOS
#elif defined BUILD_ANDROID
#else
#endif

#define STR 512.0

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], texture2d<float> u_texture2 [[texture(1)]], sampler sampler_u_texture [[sampler(0)]], sampler sampler_u_texture2 [[sampler(1)]])
{
    float4 output;
    vec4 color = u_color * texture2D(u_texture, v_texcoord);

    float kt = u_time2;
    vec2 tx = v_texcoord;
    tx.y *= 1.0 + v_position.y / pixels_y * 1.5;
    vec4 nmap1 = texture2D(u_texture2, tx+vec2(-kt,+kt)+0.25);
    vec4 nmap2 = texture2D(u_texture2, tx+vec2(+kt,-kt));
    vec4 nmap = mix(nmap1, nmap2, 0.5);
    vec3 normal = normalize(nmap.xyz);

    vec3 light = normalize(vec3(0.86, 0.75, 0.5));

    float vcoef = v_position.y / pixels_y / 1.2;

            vec3 R = normalize(reflect(-light, normal * min(color.a * 2.0, 1.0)));
        float specular = dot(R, normalize(vec3(0.0, 0.15, 0.75)));
                specular /= STR - specular * STR + specular;
        specular *= max(vcoef - 0.35, 0.0) * (1.0 - color.b) * color.a;

            color.rgb += specular;
        output = clamp(color, 0.0, 1.0);
    return output;
}
