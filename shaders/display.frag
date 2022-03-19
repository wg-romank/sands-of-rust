precision highp float;

uniform sampler2D field;
uniform sampler2D color_texture;

varying vec2 frag_uv;

vec4 cell_color(vec4 pixel) {
    vec2 color_uv = vec2(pixel.r, 0);
    return texture2D(color_texture, color_uv);
}

void main() {
    vec2 uv = vec2(frag_uv.x, frag_uv.y);
    vec4 pixel = texture2D(field, uv);
    vec4 cell_color = cell_color(pixel);

    gl_FragColor = cell_color;
}