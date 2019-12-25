
struct sPSInput
{
    float4 pos: SV_POSITION;
    float4 v_color: COLOR;
};

float4 main(sPSInput input) : SV_TARGET 
{
    return input.v_color;
}
