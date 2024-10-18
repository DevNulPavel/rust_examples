

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float3 nor : NORMAL;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    float3 light = float3(-0.62481, 0.390512, 0.781024);
	float coef = 0.85 + max(dot(input.nor, light), 0.0)*0.4;
	
    float4 color = Texture.Sample(Sampler, input.tex);
    color.rgb *= coef;

    // Final composite.
	return u_color * color;
}

