attribute PRECISION_HIGH vec3 a_position;
attribute float a_opacity;
attribute vec2 a_texCoord;

uniform PRECISION_HIGH mat4 u_transformCustom;

varying float v_opacity;
varying vec2 v_texCoord;

void main(void)
{
    gl_Position = u_transformCustom * vec4(a_position, 1.0);
    v_opacity = a_opacity;
	v_texCoord = a_texCoord;
}
