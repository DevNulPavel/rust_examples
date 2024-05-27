

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_tex_scaling;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    float2 v_texcoord = input.tex;

    float4 color = u_color * Texture.Sample(Sampler, v_texcoord);

    float tex_coord = 0.8 * v_texcoord.y / u_tex_scaling;

    color.rgb = lerp(color.rgb, float3(0.85 * tex_coord, 0.3, 0.7), tex_coord);

    return color;
}

