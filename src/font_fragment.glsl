#version 130
in vec2 v_tex_coords;
out vec4 color;

uniform float font_char_count;
uniform float scan_height;
uniform float scan_coverage;
uniform vec4 fg_color;
uniform vec4 bg_color;
uniform usampler2D font_tex;
uniform usampler2D data_tex;

void main() {

	vec2 term_pos = vec2(floor(v_tex_coords.x),floor(v_tex_coords.y));
	float c_idx = texelFetch(data_tex,ivec2(int(term_pos.x),int(term_pos.y)),0).r;
	vec2 char_off = v_tex_coords - term_pos;
    vec2 tex_pos = vec2((c_idx+char_off.x)/font_char_count, char_off.y);

	// the beam is considered to run directly through the middle of the scanline.
	// beam_off is the distance from the current pixel to the beam centerline,
	// and should range from -0.5 to 0.5.	
	float beam_off = (mod(char_off.y,scan_height)/scan_height) - 0.5;

	// the "gun" returns whether the elctron gun is on at this point in the
	// beam pass; essentially whether we're displaying a pixel.
    float gun = texture(font_tex,tex_pos).r;

    float brightness;
	if (abs(beam_off) < scan_coverage/2.0) {
		brightness = 1.0;
	} else {
		brightness = 0.2;
	}
	//float brightness = 1.0 - clamp(2.0*abs(beam_off) - scan_coverage,0.0,1.0); // temp
	if (gun <= 0.0) { brightness = brightness * 0.2; }
	brightness = pow(brightness, 2.5);

	color = mix(bg_color, fg_color, brightness);
}
