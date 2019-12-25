attribute vec4 a_position;
attribute vec4 a_normal;
attribute vec4 a_color;
attribute vec2 a_texcoord;

varying vec2 v_texcoord;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif


void main(void)
{
    v_texcoord = a_texcoord;
    gl_Position = u_transform * a_position;
}
