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
	float2 v_fragment_screen_coords : TEXCOORD1;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, float4(input.a_position[0], input.a_position[1], input.a_position[2], 1.0));
	output.v_texcoord = input.a_texcoord;
    output.v_fragment_screen_coords = (float2(output.pos[0], output.pos[1]) / output.pos[3] + 1.0) / 2.0;
    return output;
}

