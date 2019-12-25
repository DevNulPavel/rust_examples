
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
};

struct sVSInput
{
    float4 a_position : POSITION;
	float2 a_texcoord : TEXCOORD0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.a_position);
	output.tex = input.a_texcoord;
    return output;
}
