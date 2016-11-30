#version 140

in vec4 v_color;
in vec2 v_tex_coords;

uniform sampler2D texture;

out vec4 color;

void main() {
  color = v_color * texture2D(texture, v_tex_coords);
}
