

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_angle;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

#define M_PI 3.1415926535897932384626433832795

float4 main(sPSInput input) : SV_TARGET
{
	float2 v_texcoord = input.tex;
	float4 color = Texture.Sample(Sampler, v_texcoord);   
    	float2 delta = v_texcoord - float2(0.5, 0.5);
    
    	float ang = - atan2( -delta.x, delta.y );
    	ang = ang - M_PI * 2.0 * floor(ang / (M_PI * 2.0) );

	return u_color * color * float4(float(ang < u_angle), float(ang < u_angle), float(ang < u_angle), float(ang < u_angle));
}

