

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_brightness;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

#define BOTTOM_SKY_COLOUR   float4(89.0, 229.0, 239.0, 255.0)
#define TOP_SKY_COLOUR      float4(64.0, 153.0, 194.0, 255.0)

float4 main(sPSInput input) : SV_TARGET
{
    float2 v_texcoord = input.tex;

    float4 color = u_color * Texture.Sample(Sampler, v_texcoord);
    color.rgb = saturate(color.rgb * u_brightness);
    return color;
}

