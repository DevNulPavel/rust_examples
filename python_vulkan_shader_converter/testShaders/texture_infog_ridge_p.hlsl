

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
    float u_brightness;
    float csf;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float3 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    float4 color = u_color * Texture.Sample(Sampler, input.tex.xy);
	color *= saturate(max(input.tex.z - 20.0 * csf, 0.0)* 0.03 / csf);
    color.rgb = saturate(color.rgb * u_brightness);
    return color;
}

