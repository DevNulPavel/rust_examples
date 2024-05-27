cbuffer ConstantBuffer : register(b0)
{
	matrix u_transform;
};

struct sVSInput
{
    float4 pos : POSITION;
    float4 nor : NORMAL;
    float4 col : COLOR0;
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
	output.tex = input.tex;
	output.pos = mul(u_transform, input.pos);
    return output;
}

