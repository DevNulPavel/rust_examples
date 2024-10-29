

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
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
	float4 color = u_color * Texture.Sample(Sampler, input.tex);
	float3 grayColor = float3(0.299, 0.587, 0.114);
	float gray = dot(lerp(color.rgb, float3(1.0, 1.0, 1.0), 0.4), grayColor);

	float3 sepia = float3(1.1290, 1.0081, 0.8103);

	return float4(float3(gray * sepia.r, gray * sepia.g, gray * sepia.b)*color.a, color.a);
}

