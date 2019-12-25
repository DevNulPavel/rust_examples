attribute PRECISION_HIGH  vec3 a_position;
attribute vec4 a_color;

uniform PRECISION_HIGH mat4 u_transformCustom;

varying vec4 v_color;

void main(void)
{
    gl_Position = u_transformCustom * vec4(a_position, 1.0);
	v_color = a_color;
}
