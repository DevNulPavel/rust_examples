
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
};

struct sVSInput
{
    float4 pos : POSITION;
	float4 col : COLOR0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float4 col : COLOR0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);
	output.col = input.col;
    return output;
}
