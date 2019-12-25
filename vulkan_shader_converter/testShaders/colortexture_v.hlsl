
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
};

struct sVSInput
{
    float4 pos : POSITION;
	float2 tex : TEXCOORD0;
	float4 col : COLOR0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
	float4 col : COLOR0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);
	output.col = input.col;
	output.tex = input.tex;
    return output;
}
