

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
	float2 uv = input.tex;
	uv.y = 1.0 - (uv.y - (int)uv.y);
	return Texture.Sample(Sampler, uv);
}

