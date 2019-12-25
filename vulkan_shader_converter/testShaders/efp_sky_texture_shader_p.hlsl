

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float u_speed;
	float u_tex_scaling;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD0;
};

//BOTTOM_SKY_COLOUR   float4(89.0, 229.0, 239.0, 255.0) / 255.0
//TOP_SKY_COLOUR      float4(64.0, 153.0, 194.0, 255.0) / 255.0

float4 main(sPSInput input) : SV_TARGET
{
    float2 v_texcoord = input.tex;
    // Final composite.
    float4 color = lerp(float4(0.3490196, 0.8980392, 0.9372549, 1.0), float4(0.2509804, 0.6, 0.7607843, 1.0), v_texcoord.y / u_tex_scaling);
    return color;
}

