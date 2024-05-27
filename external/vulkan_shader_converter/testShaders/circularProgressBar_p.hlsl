
Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
    matrix transform;
	float4 u_progress;
	float u_inverse;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float2 tex : TEXCOORD0;
};

float4 main(sPSInput input) : SV_TARGET
{
	#define PI 3.141592653589793

	const float2 down = float2(0.0, -1.0);
	const float2 texCoordCenter = float2(0.5, 0.5);

	float2 toPixelDirection = normalize(input.tex - texCoordCenter);
	float dotValue = dot(down, toPixelDirection);

	float acosValue = acos(dotValue);
	float angleSign = sign(toPixelDirection.x);

	float angle = 1.0 - (acosValue / PI * angleSign + 1.0) / 2.0;
	if (u_inverse > 0.0) {
		angle = (acosValue / PI * angleSign + 1.0) / 2.0;
	}
	float resultMul = 1.0 - step(u_progress, angle);
	return Texture.Sample(Sampler, input.tex) * resultMul;
}
