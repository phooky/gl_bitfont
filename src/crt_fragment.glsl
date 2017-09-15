#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform float decay_factor;
uniform vec4 bg_color;
uniform sampler2D last_frame_tex;
uniform sampler2D new_beam_tex;

void main() {
	vec4 last_c = texture(last_frame_tex,v_tex_coords);
	last_c = mix(last_c,bg_color, decay_factor);
	vec4 new_c = texture(new_beam_tex,v_tex_coords);
	color = clamp(last_c + new_c, 0.0, 1.0);
}
