Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_brightness;
	float u_time;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 v_texcoord : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
	float4 color = u_color * Texture.Sample(Sampler, input.v_texcoord).a;
    return color;
}

