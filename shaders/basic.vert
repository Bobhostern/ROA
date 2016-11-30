#version 140

in vec2 position;
in vec4 color;
in vec2 tex_coords;

out vec4 v_color;
out vec2 v_tex_coords;

uniform mat4 projection;
uniform mat3 view;
uniform mat3 model;
uniform float zlayer;

void main() {
  v_tex_coords = tex_coords;
  v_color = color;
  vec3 local_pos =  view * model * vec3(position, 1);
  // Found the reason for weirdness and figured out a more logical conclusion.
  // nalgebra::Orthogonal*3 flips the Z axis, except out mat3s view and model set Z as positive,
  // making it in front of the camera, so we either negate z, or set it to 0. :)
  vec4 position = projection * vec4(local_pos.xy, 0, 1);
  gl_Position = vec4(position.xy, zlayer, 1);
}
