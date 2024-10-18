
cbuffer ConstantBuffer : register(b0)
{
	matrix u_transform;
	float u_time;
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
    float2 v_texcoord : TEXCOORD0;
    float2 v_texcoord1 : TEXCOORD1;
    float2 v_texcoord2 : TEXCOORD2;
    float v_border : TEXCOORD3;
};

sPSInput main(sVSInput input)
{

	float xmin = 0.0;
    float xmax = 14173.4;
    
    float ymin = 5.91;
    float ymax = 9046.35351;

	sPSInput output;
	output.v_texcoord2 = float2((input.pos.x) / xmax, 1.0 - (input.pos.z) / ymax);

	float v_opc = input.pos.z / ymax;

    output.v_texcoord = 0.6 * (1.0 + pow((1.0 - v_opc), 2.0)) * input.tex;
    
    output.v_texcoord1 = 0.45 * output.v_texcoord + float2(12.0, 15.0) * u_time;
    output.v_texcoord += 6.0 * u_time;
    output.v_border = 6.0 * input.nor.z;
	output.pos = mul(u_transform, input.pos);
    
	return output;
}

