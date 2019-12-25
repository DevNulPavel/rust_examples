
Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

struct sPSInput
{
    float4 pos: SV_POSITION;
    float v_opacity: COLOR0;
    float2 v_texCoord: TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET 
{   
    float4 color = Texture.Sample(Sampler, input.v_texCoord);
    return color * input.v_opacity;
}
