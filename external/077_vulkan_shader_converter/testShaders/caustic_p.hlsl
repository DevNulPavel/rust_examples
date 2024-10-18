Texture2D Texture : register(t0);
Texture2D Texture2 : register(t1);
SamplerState Sampler : register(s0);
SamplerState Sampler2 : register(s1);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float2 v_texcoord : TEXCOORD0;
	float2 v_texcoord_caustic1 : TEXCOORD1;
	float2 v_texcoord_caustic2 : TEXCOORD2;
};

float4 main(sPSInput input) : SV_TARGET
{
    float4 color = Texture.Sample(Sampler, input.v_texcoord);

    float s = (color.r + color.g + color.b) * 0.1;
    float2 sVec = float2(s, s);

    float4 colorCaustic1 = Texture2.Sample(Sampler2, input.v_texcoord_caustic1 + sVec);
    float4 colorCaustic2 = Texture2.Sample(Sampler2, input.v_texcoord_caustic2 - sVec);

    float4 colorCaustic = colorCaustic1 * colorCaustic2 * float4(1.0, 0.6, 0.2, 1.0);
    colorCaustic.rgb = clamp(colorCaustic.rgb, 0.0, 1.0);

    float y = step(input.v_texcoord.y, 0.3) * clamp((0.3 - input.v_texcoord.y) * 10.0, 0.0, 1.0);

	color.rgb += colorCaustic.rgb * y;

    return clamp(color, 0.0, 1.0);
}