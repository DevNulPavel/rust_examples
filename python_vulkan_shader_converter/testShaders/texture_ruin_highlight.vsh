attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec3 v_texcoord;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
uniform highp mat4 u_project;
uniform highp mat4 u_view;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
uniform highp mat4 u_project;
uniform highp mat4 u_view;
#else
uniform mat4 u_transform;
uniform mat4 u_project;
uniform mat4 u_view;
#endif

uniform float fogParam;

void main(void)
{
    gl_Position = u_transform * vec4(a_position.xyz, 1.0);
	v_texcoord = vec3(a_texcoord, a_position.w - fogParam);
}
