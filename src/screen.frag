#version 140
// TODO Might have to create other versions for earlier GLs

in vec2 v_tex_coords;

uniform sampler2D game;
uniform sampler2D gui;

out vec4 color;

void main() {
  vec4 mtx = texture(game, v_tex_coords);
  vec4 gtx = texture(gui, v_tex_coords);

  color = mix(mtx, gtx, gtx.a);
}
