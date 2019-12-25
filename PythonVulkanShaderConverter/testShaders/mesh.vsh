attribute vec4 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;

varying PRECISION_HIGH vec3 v_normal;
varying PRECISION_HIGH vec2 v_texcoord;

uniform PRECISION_HIGH mat4 u_transform;
uniform PRECISION_HIGH mat4 u_world;
uniform float u_flipx;

void main(void) {
    gl_Position = u_transform * vec4(a_position.x * u_flipx, a_position.yzw);
    
    v_normal = normalize(vec3(u_world * vec4(a_normal.x * u_flipx, a_normal.yz, 0.0)));

	v_texcoord = a_texcoord;
}
