
Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float2 tex : TEXCOORD0;
	float4 col : COLOR0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
    float4 color = input.col;
    color.rgb *= color.a;
    return Texture.Sample(Sampler, input.tex) * color * u_color;
}

