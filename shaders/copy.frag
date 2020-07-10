precision highp float;

uniform sampler2D field;

varying vec2 frag_uv;

void main() {
    gl_FragColor = texture2D(field, frag_uv);
}
