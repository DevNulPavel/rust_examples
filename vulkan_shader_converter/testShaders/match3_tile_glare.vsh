attribute PRECISION_HIGH vec4 a_position;
attribute PRECISION_HIGH vec2 a_texcoord;
uniform PRECISION_HIGH mat4 u_transform;
uniform bool u_flipped_y;

varying PRECISION_HIGH vec2 v_texcoord;
varying PRECISION_HIGH vec2 v_fragment_screen_coords;

void main(void)
{
    gl_Position = u_transform * a_position;
	v_texcoord = a_texcoord;
    if (u_flipped_y) {
        v_fragment_screen_coords = vec2(gl_Position.x / gl_Position.w + 1.0, 1.0 - gl_Position.y / gl_Position.w) / 2.0;
    } else {
        v_fragment_screen_coords =  (gl_Position.xy / gl_Position.w + 1.0) / 2.0;
    }
}
