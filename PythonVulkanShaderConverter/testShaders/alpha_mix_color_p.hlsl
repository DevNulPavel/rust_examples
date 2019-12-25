

Texture2D Texture : register(t0);
Texture2D Texture2 : register(t1);
SamplerState Sampler : register(s0);


cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float delta_x;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
    float4 color_mask = u_color * Texture.Sample(Sampler, input.tex);
    float2 tex = input.tex; 
    tex.x -= delta_x.x;
    float4 color = Texture2.Sample(Sampler, tex);
    color = clamp(color_mask * color.a + color_mask.a * color, 0.0, 1.0);
    return color;
}
