
struct sPSInput
{
    float4 pos: SV_POSITION;
    float4 v_color: COLOR;
};

float4 main(sPSInput input) : SV_TARGET
{
    input.v_color = float4(0.65f, 0.75f, 0.8f, 1.0f);
    return input.v_color;
}
