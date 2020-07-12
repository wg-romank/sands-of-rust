// #version 130

precision highp float;

uniform sampler2D field;
uniform sampler2D external_force;
uniform vec2 field_size;

varying vec2 frag_uv;

vec4 textureOffset(vec2 uv, vec2 offset) {
    // todo: handle borders?
    // handling with clamp currently
    return texture2D(field, (uv * field_size + offset) / field_size);
}

const int EMPTY = 0x0;
const int SAND = 0x1;

int encodeCell(vec4 contents) {
    if (contents.x > 0.0) {
        return SAND;
    } else {
        return EMPTY;
    }
}

int neighborhood(vec2 uv) {
    // need to apply mask based on own coordinates
    // instead of same pattern over whole picture
    // it must use particular parts with respect to current iteration (zero shift, shift x, shift y)
    int s1 = encodeCell(textureOffset(uv, vec2(0, 0))) << 3;
    int s2 = encodeCell(textureOffset(uv, vec2(-1, 0))) << 2;
    int s3 = encodeCell(textureOffset(uv, vec2(0, 1))) << 1;
    int s4 = encodeCell(textureOffset(uv, vec2(-1, 1)));

    return s1 | s2 | s3 | s4;
}

void main() {
    int mask = neighborhood(frag_uv);

    int next_mask = 0;
    switch(mask) {
        case 0x1000:
            next_mask = 0x0010;
            break;
        default:
            next_mask = mask;
            break;
    }

    // vec4 u = texture2D(field, frag_uv);
    // // right
    // vec4 u_1_i = textureOffset(frag_uv, vec2(-1, 0));
    // // left
    // vec4 u_i_1 = textureOffset(frag_uv, vec2(1, 0));

    // // up
    // vec4 u_j_1 = textureOffset(frag_uv, vec2(0, -1));
    // // down
    // vec4 u_1_j = textureOffset(frag_uv, vec2(0, 1));

    // vec4 F = texture2D(external_force, frag_uv);

    // GAME LOOP ITERATION HERE

    float new_u = float(next_mask);

    gl_FragColor = vec4(new_u, new_u, new_u, 1.0);
} 