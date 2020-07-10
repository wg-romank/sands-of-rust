precision highp float;

uniform sampler2D field;

varying vec2 frag_uv;

void main() {
    vec4 cellColor = texture2D(field, frag_uv);

    gl_FragColor = vec4(cellColor.rgb, 1.0);
}