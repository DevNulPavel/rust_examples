
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transformCustom;
};

struct sVSInput
{
    float3 a_position: POSITION;
    float a_opacity: COLOR0;
    float2 a_texCoord: TEXCOORD0;
};

struct sPSInput
{
    float4 pos: SV_POSITION;
    float v_opacity: COLOR0;
    float2 v_texCoord: TEXCOORD0;
};


sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transformCustom, float4(input.a_position[0], input.a_position[1], input.a_position[2], 1.0));
	output.v_opacity = input.a_opacity;
    output.v_texCoord = input.a_texCoord;
    return output;
}

