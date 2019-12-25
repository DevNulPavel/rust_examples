
cbuffer ConstantBuffer : register(b0)
{
	matrix u_transform;
	float u_flipx;
	matrix u_world;
};

struct sVSInput
{
    float4 pos : POSITION;
	float4 nor : NORMAL;
    float2 tex : TEXCOORD0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float3 nor : NORMAL;
    float2 tex : TEXCOORD0;
};

sPSInput main(sVSInput input)
{
	sPSInput output;
	output.pos = mul(u_transform, float4(input.pos.x * u_flipx, input.pos.yzw));
	float4 normal = input.nor;
	normal.x *= u_flipx;
	normal = mul(u_world, normal);
	output.nor = normalize(normal.xyz);
	output.tex = input.tex;
	return output;
}

