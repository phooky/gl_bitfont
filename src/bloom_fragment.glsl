#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D in_tex;
uniform float weights[5];
uniform vec2 offset;

void main() {
	vec3 c = vec3(0.0,0.0,0.0);
	for (int idx = 0; idx < 5; idx++) {
		vec2 coords = v_tex_coords + (offset * float(idx-2));
		float w = weights[idx];
		c = c + (texture(in_tex, coords).rgb * weights[idx]);
	}
	color = vec4(c.rgb, 1.0);
}
