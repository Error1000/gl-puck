#version 150
attribute vec3 position;
attribute vec2 tex_coord;
out vec2 pass;

uniform mat4 mvp;

void main() {
    pass = tex_coord;
    gl_Position = mvp * vec4(position, 1.0);
}