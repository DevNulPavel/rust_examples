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
    float u_progress;
    float u_inverse;
};

// Varyings:
struct ColorInOut {
    float4 pos [[position]];
    vec2 v_texcoord;
};

// Uniforms defines:
#define u_progress uniforms.u_progress
#define u_inverse uniforms.u_inverse

// Varying defines:
#define v_texcoord input.v_texcoord
#define PI 3.141592653589793

// Main function
fragment float4 fragFunc(ColorInOut input [[stage_in]], constant ConstantBuffer& uniforms [[buffer(1)]], texture2d<float> u_texture [[texture(0)]], sampler sampler_u_texture [[sampler(0)]]) {
    float4 output;
     const vec2 down = vec2(0.0, -1.0);
     const vec2 texCoordCenter = vec2(0.5, 0.5);

     vec2 toPixelDirection = normalize(v_texcoord - texCoordCenter);
     float dotValue = dot(down, toPixelDirection);

          PRECISION_HIGH float acosValue = acos(dotValue);
     PRECISION_LOW float angleSign = sign(toPixelDirection.x);

     float angle = 1.0 - (acosValue / PI * angleSign + 1.0) / 2.0;
     if (u_inverse > 0.0){
          angle = (acosValue / PI * angleSign + 1.0) / 2.0;
     }
          PRECISION_LOW float resultMul = 1.0 - step(u_progress, angle);
     output = texture2D(u_texture, v_texcoord) * resultMul;

                            return output;
}

