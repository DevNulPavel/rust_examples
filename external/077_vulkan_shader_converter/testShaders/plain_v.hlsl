
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
};

struct sVSInput
{
    float4 pos : POSITION;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);
    return output;
}

