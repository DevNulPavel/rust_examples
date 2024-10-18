
struct sPSInput
{
    float4 pos : SV_POSITION;
	float4 col : COLOR0;
};

float4 main(sPSInput input) : SV_TARGET
{
    // Final composite.
    return input.col;
}
