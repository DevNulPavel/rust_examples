

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float u_param_x;
	float u_param_y;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
	float2 v_texcoord = input.tex;
	float4 color = Texture.Sample(Sampler, v_texcoord);

	float x = abs((v_texcoord.x - 0.5) / 0.5) - u_param_x;
	float y = abs((v_texcoord.y - 0.5) / 0.5) - u_param_y;
    	color.a = min(color.a, clamp(max(y, x) * 10000.0, 0.0, 1.0));
	color.rgb *= color.a;
	return color;
}

