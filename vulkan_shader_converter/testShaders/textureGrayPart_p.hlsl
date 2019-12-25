

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_gray_part;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
	float4 color = Texture.Sample(Sampler, input.tex).bgra;
	float gray = dot(color.rgb, float3(0.299, 0.587, 0.114));
    return u_color * (u_gray_part * float4(gray, gray, gray, color.a) + (1.0 - u_gray_part) * color);
}

