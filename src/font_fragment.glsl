#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform float font_char_count;
uniform float scan_height;
uniform float scan_coverage;
uniform vec4 in_color;
uniform usampler2D font_tex;
uniform usampler2D data_tex;

void main() {

	vec2 term_pos = vec2(floor(v_tex_coords.x),floor(v_tex_coords.y));
	float c_idx = texelFetch(data_tex,ivec2(int(term_pos.x),int(term_pos.y)),0).r;

	vec2 char_off = v_tex_coords - term_pos;
	// Skip bits outside the scanline
	if ( mod(char_off.y,scan_height)  > scan_coverage*scan_height) {
		color = vec4(0.0,0.0,0.2,1.0);
		return;
	}
	vec2 tex_pos = vec2((c_idx+char_off.x)/font_char_count, char_off.y);
    float rv = texture(font_tex,tex_pos).r;
    if (rv > 0.0) {
        color = in_color;
    } else {
    	color = vec4(0.0,0.0,0.3,1.0);
    }
}
