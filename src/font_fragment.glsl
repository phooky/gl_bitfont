#version 130
in vec2 v_tex_coords;
out vec4 color;

// uniform vec2 cell_size; // for now fake it // everything is an 'a' (97)
uniform vec4 in_color;
uniform usampler2D tex;

// Compiling in display size
uniform uint data[80*24];

void main() {
	vec2 cell_size = vec2(8.0,10.0);
	float max_char = 128.0;

	vec2 term_pos = vec2(floor(v_tex_coords.x),floor(v_tex_coords.y));
	uint idx = 80u * uint(term_pos.y) + uint(term_pos.x);
	float char_idx = float( (data[idx/4u] >> (idx%4u)*8u) & 255u );

	vec2 char_off = v_tex_coords - term_pos;
	vec2 tex_pos = vec2((char_idx+char_off.x)/max_char, char_off.y);
    float rv = texture(tex,tex_pos).r;
    //float rv = texture(tex,vec2(v_tex_coords.x/80.0,v_tex_coords.y/24.0)).r;
    if (rv > 0.0) {
        color = vec4(0.0,1.0,0.0, 1.0);
    } else {
        color = vec4(0.0,0.0,0.2,1.0);
    }
}
