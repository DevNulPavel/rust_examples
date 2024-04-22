attribute vec4 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;
//attribute vec4 a_color;

varying vec3 v_normal;
varying vec4 v_color;
varying vec2 v_texcoord;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif
uniform mat4 u_world;

//uniform mat4 u_skinmatrix1;
//uniform mat4 u_skinmatrix2;
//uniform mat4 u_skinmatrix3;
//uniform mat4 u_skinmatrix4;

void main(void)
{
// 	vec4 pos = (u_skinmatrix1 * a_position) * a_color.x;
//	pos += (u_skinmatrix2 * a_position) * a_color.y;
//	pos += (u_skinmatrix3 * a_position) * a_color.z;
//	pos += (u_skinmatrix4 * a_position) * a_color.w;

    gl_Position = u_transform * a_position;
    
    v_normal = normalize(vec3(u_world * vec4(a_normal,0.0)));

    //v_color = a_color;
	v_texcoord = a_texcoord;
}
