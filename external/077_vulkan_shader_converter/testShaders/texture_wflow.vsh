attribute vec4 a_position;
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
    gl_Position = u_transform * vec4(a_position.xyz, 1.0);
	v_texcoord = a_texcoord;
}
