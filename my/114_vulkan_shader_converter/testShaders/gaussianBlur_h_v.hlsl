
cbuffer ConstantBuffer : register(b0)
{
	matrix u_transform;
	float hPxSize;
};

struct sVSInput
{
    float4 pos : POSITION;
    float2 tex : TEXCOORD0;
};

struct sPSInput
{
    float4 pos : SV_POSITION;
	float4 v_blurTexCoords[7] : TEXCOORD1;
};

sPSInput main(sVSInput input)
{
    sPSInput output;
	output.pos = mul(u_transform, input.pos);

	output.v_blurTexCoords[0].xy = input.tex + float2(-hPxSize * 7.0, 0.0);
	output.v_blurTexCoords[0].zw = input.tex + float2(-hPxSize * 6.0, 0.0);
	output.v_blurTexCoords[1].xy = input.tex + float2(-hPxSize * 5.0, 0.0);
	output.v_blurTexCoords[1].zw = input.tex + float2(-hPxSize * 4.0, 0.0);
	output.v_blurTexCoords[2].xy = input.tex + float2(-hPxSize * 3.0, 0.0);
	output.v_blurTexCoords[2].zw = input.tex + float2(-hPxSize * 2.0, 0.0);
	output.v_blurTexCoords[3].xy = input.tex + float2(-hPxSize, 0.0);
	output.v_blurTexCoords[3].zw = input.tex + float2(hPxSize, 0.0);
	output.v_blurTexCoords[4].xy = input.tex + float2(hPxSize * 2.0, 0.0);
	output.v_blurTexCoords[4].zw = input.tex + float2(hPxSize * 3.0, 0.0);
	output.v_blurTexCoords[5].xy = input.tex + float2(hPxSize * 4.0, 0.0);
	output.v_blurTexCoords[5].zw = input.tex + float2(hPxSize * 5.0, 0.0);
	output.v_blurTexCoords[6].xy = input.tex + float2(hPxSize * 6.0, 0.0);
	output.v_blurTexCoords[6].zw = input.tex + float2(hPxSize * 7.0, 0.0);

    return output;
}

