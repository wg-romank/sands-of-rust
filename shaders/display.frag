precision highp float;

uniform sampler2D field;
uniform sampler2D color_texture;

varying vec2 frag_uv;

float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

vec4 cell_color(vec4 pixel) {
    vec2 color_uv = vec2(pixel.r, 0);
    vec4 color = texture2D(color_texture, color_uv);
    float scale = mix(0.7, 0.9, rand(frag_uv));
    return vec4(scale * color.rgb, 1.);
}

void main() {
    vec2 uv = vec2(frag_uv.x, frag_uv.y);
    vec4 pixel = texture2D(field, uv);
    vec4 cell_color = cell_color(pixel);

    gl_FragColor = cell_color;
}