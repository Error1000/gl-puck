#version 150
uniform sampler2D obj_tex;
in vec2 pass;

out vec4 out_color;

void main() {
    vec4 c = texture2D(obj_tex, pass);
    if(c.a != 1.0) discard;
    else out_color = c;
}