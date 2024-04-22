
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
    float u_speed;
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

    float2 flow;
    flow.x = u_speed;
    flow.y = 0.0;

    output.tex = input.tex + frac(flow.xy);

    return output;
}
