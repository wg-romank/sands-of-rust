precision highp float;

uniform sampler2D field;

varying vec2 frag_uv;

const float EMPTY = 10.;
const float WATER = 20.;
const float SAND = 30.;

const vec4 error = vec4(1., 0., 0., 1.);

float encodeCell(vec4 contents) {
    return contents.x * 255.;
}

// TODO: cannot use exact match here on mobile :(
vec4 cellColor(vec4 pixel) {
    float cellType = encodeCell(pixel);
    if (abs(cellType - EMPTY) < 1.) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else if (abs(cellType - SAND) < 1.) {
        return vec4(vec3(168, 134, 42) / 255.0, 1.0);
    } else if (abs(cellType - WATER) < 1.) {
        return vec4(vec3(103, 133, 193) / 255.0, 1.0);
    } else {
        return error;
    }
}

void main() {
    vec2 uv = vec2(frag_uv.x, frag_uv.y);
    vec4 pixel = texture2D(field, uv);
    vec4 cell_color = cellColor(pixel);

    gl_FragColor = cell_color;
}