

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

cbuffer ConstantBuffer : register(b0)
{
};

struct sPSInput
{
    float4 pos : SV_POSITION;
    float4 v_blurTexCoords[7] : TEXCOORD1;
};

float4 main(sPSInput input) : SV_TARGET
{
	float4 color = Texture.Sample(Sampler, (input.v_blurTexCoords[3].xy + input.v_blurTexCoords[3].zw) * 0.5)*0.159576912161;

	color += Texture.Sample(Sampler, input.v_blurTexCoords[0].xy)*0.0044299121055113265;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[0].zw)*0.00895781211794;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[1].xy)*0.0215963866053;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[1].zw)*0.0443683338718;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[2].xy)*0.0776744219933;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[2].zw)*0.115876621105;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[3].xy)*0.147308056121;

    color += Texture.Sample(Sampler, input.v_blurTexCoords[3].zw)*0.147308056121;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[4].xy)*0.115876621105;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[4].zw)*0.0776744219933;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[5].xy)*0.0443683338718;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[5].zw)*0.0215963866053;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[6].xy)*0.00895781211794;
    color += Texture.Sample(Sampler, input.v_blurTexCoords[6].zw)*0.0044299121055113265;

    return color;
}

