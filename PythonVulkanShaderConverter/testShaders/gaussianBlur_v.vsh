attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec2 v_texcoord;
varying vec4 v_blurTexCoords[7];

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif

uniform float vPxSize;

void main(void)
{
    gl_Position = u_transform * a_position;
	v_texcoord = a_texcoord;

    v_blurTexCoords[ 0].xy = v_texcoord + vec2(0.0, -vPxSize * 7.0);
    v_blurTexCoords[ 0].zw = v_texcoord + vec2(0.0, -vPxSize * 6.0);
    v_blurTexCoords[ 1].xy = v_texcoord + vec2(0.0, -vPxSize * 5.0);
    v_blurTexCoords[ 1].zw = v_texcoord + vec2(0.0, -vPxSize * 4.0);
    v_blurTexCoords[ 2].xy = v_texcoord + vec2(0.0, -vPxSize * 3.0);
    v_blurTexCoords[ 2].zw = v_texcoord + vec2(0.0, -vPxSize * 2.0);
    v_blurTexCoords[ 3].xy = v_texcoord + vec2(0.0, -vPxSize);
    v_blurTexCoords[ 3].zw = v_texcoord + vec2(0.0,  vPxSize);
    v_blurTexCoords[ 4].xy = v_texcoord + vec2(0.0,  vPxSize * 2.0);
    v_blurTexCoords[ 4].zw = v_texcoord + vec2(0.0,  vPxSize * 3.0);
    v_blurTexCoords[5].xy = v_texcoord + vec2(0.0,  vPxSize * 4.0);
    v_blurTexCoords[5].zw = v_texcoord + vec2(0.0,  vPxSize * 5.0);
    v_blurTexCoords[6].xy = v_texcoord + vec2(0.0,  vPxSize * 6.0);
    v_blurTexCoords[6].zw = v_texcoord + vec2(0.0,  vPxSize * 7.0);
    
    
    
    
}
