

Texture2D Texture : register(t0);
Texture2D Texture2 : register(t1);
SamplerState Sampler : register(s0);
SamplerState Sampler2 : register(s1);

cbuffer ConstantBuffer : register(b0)
{
	float u_time;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
    float4 flow = Texture2.Sample(Sampler2, input.tex);
    flow.xy = (flow.xy - (1.0 / (255.0 / 128.0))) * 2.0 / (flow.w * 255.0) * 80.0;
    flow.y *= -1.0;
    flow.x *= -1.0;

    float time = u_time * 2.0 * 100.0;
    time = time - floor(time);

    float time2 = (time + 0.5);
    time2 = time2 - floor(time2);

    float4 tex1 = Texture.Sample(Sampler, input.tex+flow.xy*time);
    float4 tex2 = Texture.Sample(Sampler, input.tex+flow.xy*time2);
    
    return lerp(tex1, tex2, abs(time * 2.0 - 1.0));
}

