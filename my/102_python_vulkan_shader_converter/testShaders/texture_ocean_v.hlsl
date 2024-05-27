
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
	float u_time;
	float topscale;
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
    float timesin : TEXCOORD1;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);
	output.tex = input.tex;
	output.timesin = sin(3.1415 * (frac(u_time * 100.0)) * 2.0) * topscale * 0.2;
    return output;
}

