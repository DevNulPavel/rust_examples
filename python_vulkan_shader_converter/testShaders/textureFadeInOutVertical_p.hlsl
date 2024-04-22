

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
    float u_edgebottom;
    float u_edgetop;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    float4 tex1 = Texture.Sample(Sampler, input.tex);
    return tex1 * min(smoothstep(0.0, u_edgebottom, input.tex.y), 1.0 - smoothstep(u_edgetop, 1.0, input.tex.y));
}

