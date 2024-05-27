attribute vec4 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;
//attribute vec4 a_color;

varying vec3 v_normal;
varying float v_alpha;
varying vec2 v_texcoord;

#if defined BUILD_IOS
uniform highp mat4 u_transform;
#elif defined BUILD_ANDROID
uniform highp mat4 u_transform;
#else
uniform mat4 u_transform;
#endif
uniform mat4 u_world;
uniform float u_flipx;

void main(void)
{
    gl_Position = u_transform * vec4(a_position.x * u_flipx, a_position.yzw);
    
    v_normal = normalize(vec3(u_world * vec4(a_normal.xyz, 0.0)));

    if (a_position.y < 0.0) {
        v_alpha = 0.5;
    } else {
        v_alpha = 1.0;
    }
	v_texcoord = a_texcoord;
}
