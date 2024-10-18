attribute vec4 a_position;
attribute vec2 a_texcoord;
uniform mat4 u_transform;
varying vec2 v_texcoord;
void main(void)
{
  gl_Position = (u_transform * a_position);
  v_texcoord = a_texcoord;
}

