Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
	float4 u_color;
	float u_brightness;
	float u_time;
	float u_height_correction_factor;
	float u_argument_addition;
	float u_angle_k;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float2 v_texcoord : TEXCOORD0;
    float2 v_fragment_screen_coords : TEXCOORD1;
};

float4 main(sPSInput input) : SV_TARGET
{
	float4 color = Texture.Sample(Sampler, input.v_texcoord) * u_color * u_brightness;
    float y = u_angle_k * (input.v_fragment_screen_coords.x - u_argument_addition);
    float dy = abs(y - input.v_fragment_screen_coords.y);

    float glow_length = 0.2 * u_height_correction_factor;
    float line_length = 0.02 * u_height_correction_factor;
    float line_highlight_strength = 0.68;
    float glow_strength = 0.4;
    float glow_modifier = clamp((exp( (-dy / glow_length) / 0.5)), 0.0, 1.0);

    color.rgb += color.rgb * (step(dy, line_length) * line_highlight_strength + glow_modifier * glow_strength) * color.a;

    return color;
}