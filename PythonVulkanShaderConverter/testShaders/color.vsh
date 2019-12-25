attribute vec4 a_position;
attribute vec4 a_color;

varying vec4 v_color;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif

void main(void)
{
    gl_Position = u_transform * a_position;
	v_color = a_color;
}
