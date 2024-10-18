
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
    float4 u_color;
    float u_time;
    float u_brightness;
};

struct sVSInput
{
    float4 pos : POSITION;
    float2 tex : TEXCOORD0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
    output.pos = mul(u_transform, input.pos);
    output.tex = input.tex;
    return output;
}

