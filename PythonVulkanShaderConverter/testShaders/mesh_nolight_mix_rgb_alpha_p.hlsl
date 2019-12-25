

Texture2D Texture : register(t0);
Texture2D Texture2 : register(t1);
SamplerState Sampler : register(s0);


cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
    float4 color = Texture.Sample(Sampler, input.tex);
    float4 color_mask = Texture2.Sample(Sampler, input.tex);
    color.a = color_mask.r;
    color *= u_color;
    return color;
}
