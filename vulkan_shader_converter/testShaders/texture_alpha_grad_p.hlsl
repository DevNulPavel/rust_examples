Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
    float4 u_color;
    float u_grad_param;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    return u_color * Texture.Sample(Sampler, input.tex) * clamp((u_grad_param - input.tex.y) * 10.0, 0.0, 1.0);
}