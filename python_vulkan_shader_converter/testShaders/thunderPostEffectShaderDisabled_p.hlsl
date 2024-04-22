Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
    float u_currentValue;
};

struct sPSInput
{
    float4 pos: SV_POSITION;
    float2 v_texcoord: TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET 
{
    float4 color = Texture.Sample(Sampler, input.v_texcoord);
    return color - float4(0.1, 0.1, 0.0, 0.0) * u_currentValue;
}