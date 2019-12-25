
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
	matrix u_view;
	float fogParam;
};

struct sVSInput
{
    float4 pos : POSITION;
    float2 tex : TEXCOORD0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float3 tex : TEXCOORD0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
    float4 p = float4(input.pos.x, input.pos.y, input.pos.z, 1.0);
    output.pos = mul(u_transform, p);
    output.tex = float3(input.tex, input.pos.w - fogParam);
    return output;
}

