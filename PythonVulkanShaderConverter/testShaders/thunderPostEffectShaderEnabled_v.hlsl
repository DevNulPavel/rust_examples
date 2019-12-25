struct sVSInput
{
    float2 a_position: POSITION;
    float2 a_texcoord: TEXCOORD0;
};

struct sPSInput
{
    float4 pos: SV_POSITION;
    float2 v_texcoord: TEXCOORD0;
};


sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = float4(input.a_position, 0.0, 1.0);
	output.v_texcoord = input.a_texcoord;
    return output;
}