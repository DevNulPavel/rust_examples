

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);


struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
    float4 v_color : COLOR0;
    float4 v_darken : COLOR1;
};

float4 main(sPSInput input) : SV_TARGET
{
	float4 color =  Texture.Sample(Sampler, input.tex);
    return lerp(input.v_darken, color, input.v_color.r) * input.v_color.a;
}

