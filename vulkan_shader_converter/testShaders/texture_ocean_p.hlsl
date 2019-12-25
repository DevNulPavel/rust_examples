

Texture2D Texture : register(t0);
Texture2D Texture2 : register(t1);
SamplerState Sampler : register(s0);
SamplerState Sampler2 : register(s1);


struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
    float timesin : TEXCOORD1;
};

float4 main(sPSInput input) : SV_TARGET
{
    float4 flow = Texture2.Sample(Sampler2, input.tex);
	float str = 1.0 / (flow.w * 255.0) * input.timesin / 3.0;
	flow.xy = (flow.xy * 2.0 - 1.0) * str;

	return Texture.Sample(Sampler, input.tex + flow.xy);
}

