precision highp float;

uniform sampler2D field;

varying vec2 frag_uv;

const int EMPTY = 0x0;
const int SAND = 0x1;
const int WATER = 0x2;

const vec4 error = vec4(1., 0., 0., 1.);

int encodeCell(vec4 contents) {
    float v = contents.x * 255.;
    if (v == 0.) {
        return EMPTY;
    } else if (v == 1.) {
        return SAND;
    } else if (v == 2.) {
        return WATER;
    } else {
        return -1;
    }
}

vec4 cellColor(int cellType) {
    if (cellType == EMPTY) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else if (cellType == SAND) {
        return vec4(vec3(168, 134, 42) / 255.0, 1.0);
    } else if (cellType == WATER) {
        return vec4(vec3(103, 133, 193) / 255.0, 1.0);
    } else {
        return error;
    }
}

void main() {
    gl_FragColor = cellColor(encodeCell(texture2D(field, vec2(frag_uv.x, 1.0 - frag_uv.y))));
}