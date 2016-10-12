// There is no transformation matrix. This is simply an alpha aware blit to the screen.
#version 330 core

in vec2 position;
in vec2 tex_coords;
// We don't care about color

out vec2 v_tex_coords;

void main() {
  v_tex_coords = tex_coords;
  gl_Position = vec4(position, 0, 1);
}
