
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
	float4 u_color;
	float u_time;
    float u_brightness;
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
    float4 p = float4(input.pos.x, input.pos.y, input.pos.z, 1.0);
	output.pos = mul(u_transform, p);
	output.tex = input.tex;
    return output;
}

