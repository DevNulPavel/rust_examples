
cbuffer ConstantBuffer : register(b0)
{
	matrix u_transform;
	matrix u_world;
	float u_flipx;
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
    float alpha : TEXCOORD1;
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

    if (input.pos.y < 0.0) {
        output.alpha = 0.5;
    } else {
        output.alpha = 1.0;
    }

	return output;
}

