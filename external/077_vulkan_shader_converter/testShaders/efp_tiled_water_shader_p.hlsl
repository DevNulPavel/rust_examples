

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

//SKY_COLOUR (64.0, 153.0, 194.0, 255.0) / 255.0

float4 main(sPSInput input) : SV_TARGET
{
    float2 v_texcoord = input.tex;

    float4 color = u_color * Texture.Sample(Sampler, v_texcoord);

    float tex_coord = v_texcoord.y / u_tex_scaling;

    color.rgb = lerp(color.rgb, float3(0.3490196, 0.8980392, 0.9372549) * tex_coord, tex_coord);
    return color;
}

