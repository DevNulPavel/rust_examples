cbuffer ConstantBuffer : register(b0)
{
    matrix u_transform;
	float u_time;
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
	float2 v_texcoord_caustic1 : TEXCOORD1;
	float2 v_texcoord_caustic2 : TEXCOORD2;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
    float4 p = float4(input.a_position.x, input.a_position.y, input.a_position.z, 1.0);
	output.pos = mul(u_transform, p);	
	output.v_texcoord = input.a_texcoord;
    
    float speed = 6.0;
    output.v_texcoord_caustic1 = input.a_texcoord * 2.5 + speed * float2(5.0 * u_time, 6.0 * u_time);
    output.v_texcoord_caustic2 = input.a_texcoord * 3.0 - speed * float2(4.0 * u_time, 8.0 * u_time);
	
    return output;
}

