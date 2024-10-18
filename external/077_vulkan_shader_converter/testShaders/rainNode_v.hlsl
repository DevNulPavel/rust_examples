
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transformCustom;
};

struct sVSInput
{
    float3 a_position: POSITION;
    float4 a_color: COLOR0;
};

struct sPSInput
{
    float4 pos: SV_POSITION;
    float4 v_color: COLOR0;
};


sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transformCustom, float4(input.a_position[0], input.a_position[1], input.a_position[2], 1.0));
	output.v_color = input.a_color;
    return output;
}

