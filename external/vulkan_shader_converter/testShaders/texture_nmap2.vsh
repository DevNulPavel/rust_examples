attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec2 v_texcoord;
varying vec4 v_worldpos;
varying vec4 v_position;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif
uniform mat4 world;

void main(void)
{
    gl_Position = u_transform * a_position;
    v_worldpos = world * a_position;
	v_texcoord = a_texcoord;
    v_position = a_position;
}
