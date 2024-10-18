attribute PRECISION_HIGH vec4 a_position;
attribute PRECISION_HIGH vec2 a_texcoord;
varying PRECISION_HIGH vec2 v_texcoord;

varying PRECISION_HIGH vec2 v_texcoord_caustic1;
varying PRECISION_HIGH vec2 v_texcoord_caustic2;

uniform PRECISION_HIGH mat4 u_transform;

uniform PRECISION_HIGH float u_time;

void main(void)
{
    gl_Position = u_transform * vec4(a_position.xyz, 1.0);
	v_texcoord = a_texcoord;
    
    PRECISION_HIGH float speed = 6.0;
    v_texcoord_caustic1 = a_texcoord * 2.5 + speed * vec2(5.0 * u_time, 6.0 * u_time);
    v_texcoord_caustic2 = a_texcoord * 3.0 - speed * vec2(4.0 * u_time, 8.0 * u_time);
}
