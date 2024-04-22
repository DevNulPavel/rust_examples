
cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
	matrix u_world;
	float u_time2;
	float4 u_color;
};

struct sVSInput
{
    float4 pos : POSITION;
    float2 tex : TEXCOORD0;
    float4 col : COLOR0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
    float4 v_color : COLOR0;
    float4 v_darken : COLOR1;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);

	float param = u_time2 + (input.pos.x + input.pos.y) * 0.0025;
    float trigonometryMult = 0.05;
    output.tex = float2(input.tex.x + sin(param) * trigonometryMult, input.tex.y + cos(param) * trigonometryMult);
    output.v_color = input.col;
    output.v_darken = lerp(float4(0.0, 0.0, 0.0, 0.0), u_color, input.col.a);
    return output;

}

