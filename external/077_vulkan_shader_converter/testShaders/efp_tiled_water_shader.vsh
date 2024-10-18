attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec2 v_texcoord;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
uniform highp float u_time;
uniform highp float u_speed;
#define FLOAT highp float
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
uniform highp float u_time;
uniform highp float u_speed;
#define FLOAT highp float
#else
uniform mat4 u_transform;
uniform float u_time;
uniform float u_speed;
#define FLOAT float
#endif

uniform float u_tex_scaling;

void main(void)
{
    gl_Position = u_transform * a_position;
    
    vec2 flow;
    flow.x = u_speed;
    flow.y = 0.0;
    v_texcoord = a_texcoord + fract(flow.xy);
}
