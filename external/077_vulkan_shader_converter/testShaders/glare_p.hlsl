Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float brightness;
	float i_radius;
	float o_radius;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 v_texcoord : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
	float4 color = u_color * Texture.Sample(Sampler, input.v_texcoord);
    float l = length(float2(0.5, 0.5) - input.v_texcoord);
 
    float4 colorBr = lerp(color, float4(1.0, 1.0, 1.0, 1.0) * color.a , brightness);
    color = lerp(colorBr, color, smoothstep(i_radius, o_radius, l));

    return color;
}

