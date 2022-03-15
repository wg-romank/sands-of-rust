precision highp float;

uniform sampler2D field;

varying vec2 frag_uv;

const vec4 error = vec4(1., 0., 0., 1.);

float unpack_cell(vec4 contents) {
    return contents.x * 255.;
}

// TODO: cannot use exact match here on mobile :(
vec4 cell_color(vec4 pixel) {
    float cellType = unpack_cell(pixel);
    float intensity = mix(0.4, 1., pixel.x);
    if (cellType == 0.) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else if (cellType > 0.) {
        return vec4(intensity * vec3(168, 134, 42) / 255.0, 1.0);
    } else {
        return error;
    }
}

void main() {
    vec2 uv = vec2(frag_uv.x, frag_uv.y);
    vec4 pixel = texture2D(field, uv);
    vec4 cell_color = cell_color(pixel);

    gl_FragColor = cell_color;
}