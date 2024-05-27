#if defined BUILD_IOS
#define PLATFORM_PRECISION highp
#elif defined BUILD_ANDROID
#define PLATFORM_PRECISION highp
#else
#define PLATFORM_PRECISION
#endif

attribute vec4 a_position;
attribute vec2 a_texcoord;
attribute vec4 a_color;

varying PLATFORM_PRECISION vec2 v_texcoord;
varying vec4 v_color;
varying vec4 v_darken;

uniform PLATFORM_PRECISION mat4 u_transform;
uniform PLATFORM_PRECISION float u_time2;

uniform vec4 u_color;

void main(void)
{
    PLATFORM_PRECISION float param = u_time2 + (a_position.x + a_position.y) * 0.0025;
    PLATFORM_PRECISION float sin_p = sin(param);
    PLATFORM_PRECISION float cos_p = cos(param);
    float trigonometryMult = 0.05;
    v_texcoord = vec2(a_texcoord.x + sin_p * trigonometryMult, a_texcoord.y + cos_p * trigonometryMult);
    v_color = a_color;
    v_darken = mix(vec4(0.0, 0.0, 0.0, 0.0), u_color, v_color.a);
    gl_Position = u_transform * a_position;
}
