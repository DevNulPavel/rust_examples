attribute PRECISION_HIGH vec4 a_position;
attribute PRECISION_HIGH vec2 a_texcoord;

uniform PRECISION_HIGH mat4 u_transform;
uniform PRECISION_HIGH float u_time;
uniform PRECISION_HIGH  float topscale;

varying PRECISION_HIGH vec2 v_texcoord;
varying PRECISION_HIGH float v_timesin;

void main(void)
{
    gl_Position = u_transform * a_position;
	v_texcoord = a_texcoord;
    v_timesin = sin(3.1415 * (fract(u_time * 100.0)) * 2.0) * topscale * 0.2;
}
