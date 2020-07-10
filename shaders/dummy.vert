precision highp float;

attribute vec2 vert_position;
attribute vec2 vert_uv;

varying vec2 frag_uv;

void main() {
    gl_Position = vec4(vert_position, 0.0, 1.0);
    frag_uv = vert_uv;
}