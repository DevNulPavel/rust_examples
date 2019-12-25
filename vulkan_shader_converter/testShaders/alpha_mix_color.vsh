attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec2 v_texcoord;
varying vec2 tc;
uniform float u_time;
uniform float delta_x;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#elif defined BUILD_TIZEN
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif

void main(void)
{
    gl_Position = u_transform * a_position;
    v_texcoord = a_texcoord;
    tc = v_texcoord;
    tc.x -= delta_x;
}
