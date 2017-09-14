#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform vec4 in_color;
uniform usampler2D font_tex;
uniform usampler2D data_tex;

void main() {
	vec2 cell_size = vec2(8.0,10.0);
	float max_char = 128.0;

	vec2 term_pos = vec2(floor(v_tex_coords.x),floor(v_tex_coords.y));
	float c_idx = texelFetch(data_tex,ivec2(int(term_pos.x),int(term_pos.y)),0).r;

	vec2 char_off = v_tex_coords - term_pos;
	vec2 tex_pos = vec2((c_idx+char_off.x)/max_char, char_off.y);
    float rv = texture(font_tex,tex_pos).r;
    if (rv > 0.0) {
        color = vec4(0.0,1.0,0.0, 1.0);
    } else {
        color = vec4(0.0,0.0,0.2,1.0);
    }
}
