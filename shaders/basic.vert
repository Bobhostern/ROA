#version 330 core

in vec2 position;
in vec4 color;

out vec4 v_color;

uniform mat4 projection;
uniform mat3 view;
uniform mat3 model;

void main() {
  // v_tex_coords = tex_coords;
  v_color = color;
  vec3 local_pos =  view * model * vec3(position, 1);
  // Found the reason for weirdness and figured out a more logical conclusion.
  // nalgebra::Orthogonal*3 flips the Z axis, except out mat3s view and model set Z to 1,
  // makeing it in front of the camera, so we either negate z, or set it to 0. :)
  gl_Position = projection * vec4(local_pos.xy, -local_pos.z, 1);
}
