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
    float2 v_texcoord : TEXCOORD0;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
    float4 p = float4(input.a_position.x, input.a_position.y, input.a_position.z, 1.0);
	output.pos = mul(u_transform, p);
	output.v_texcoord = input.a_texcoord;
    return output;
}