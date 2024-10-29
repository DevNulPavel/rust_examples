

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);


Texture2D Texture2 : register(t1);
SamplerState Sampler2 : register(s1);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 v_texcoord : TEXCOORD0;
    float2 v_texcoord1 : TEXCOORD1;
    float2 v_texcoord2 : TEXCOORD2;
    float v_border : TEXCOORD3;
};

float4 main(sPSInput input) : SV_TARGET
{

        float4 waterColor1 = Texture.Sample(Sampler, input.v_texcoord);
        float4 waterColor2 = Texture.Sample(Sampler, input.v_texcoord1);
        float4 color2 = Texture2.Sample(Sampler2, input.v_texcoord2);

        float4 waterColor = (waterColor1 + waterColor2);
        float foam = clamp(1.25 * (color2.g - 0.2), 0.0, 1.0);
        float opacity = color2.r - foam * 0.3;
        float light = color2.b;

        float4 colorNew = lerp(float4(0.0, 0.5, 0.8, opacity), waterColor1, 0.3);
        float4 color = lerp(float4(0.35, 0.9, 0.55, opacity), colorNew, opacity);

        float wSum = (waterColor.r + waterColor.g + waterColor.b);
        float wFoam = wSum * (wSum - 3.2);

        //wFoam = (clamp(wFoam * (0.25 - foam), 0.0, 1.0));
        float border = clamp(input.v_border, 0.0, 1.0);
        wFoam = (clamp(wFoam * 0.275 + foam * 2.0 * border, 0.0, 1.0));

        color = lerp(color, float4(1.0, 1.0, 1.0, 1.0), wFoam); // пенка
        color = lerp(color, float4(0.376, 0.894, 0.901, 1.0), light);
        color.a = lerp(color.a, 0.0, border);

        color.rgb *= color.a;
        return color;
}

