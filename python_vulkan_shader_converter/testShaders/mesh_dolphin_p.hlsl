

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
    float alpha : TEXCOORD1;
};

float4 main(sPSInput input) : SV_TARGET
{
    float3 light = normalize(float3(-1.0, 0.5, 1.0));
	float coef = max(dot(normalize(input.nor), light), 0.50);
    float4 color = Texture.Sample(Sampler, input.tex);
    color.rgb *= coef * 1.33;
    color.a *= input.alpha;

    // Final composite.
	return u_color * color;
}

