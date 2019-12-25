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

float4 main(sPSInput input) : SV_TARGET
{
    float4 color = u_color * Texture.Sample(Sampler, input.tex);
    color.rgb = clamp((color.rgb * u_brightness + float3(0.05, 0.07, 0.06) * color.a), 0.0, 1.0);
    return color;
}

