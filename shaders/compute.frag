precision highp float;

uniform sampler2D field;
uniform sampler2D external_force;
uniform vec2 field_size;

varying vec2 frag_uv;

vec4 pack(float depth) {
    const vec4 bitSh = vec4(256.0 * 256.0 * 256.0,
                       256.0 * 256.0,
                       256.0,
                      1.0);
    const vec4 bitMsk = vec4(0,
                         1.0 / 256.0,
                         1.0 / 256.0,
                             1.0 / 256.0);
    vec4 comp = fract(depth * bitSh);
    comp -= comp.xxyz * bitMsk;
    return comp;
}

float unpack(vec4 packedZValue) {
    const vec4 bitShifts = vec4(1.0 / (256.0 * 256.0 * 256.0),
                        1.0 / (256.0 * 256.0),
                        1.0 / 256.0,
                        1);
    float shadow = dot(packedZValue , bitShifts);
    return shadow;
}

vec4 textureOffset(vec2 uv, vec2 offset) {
    // todo: handle borders?
    // handling with clamp currently
    return texture2D(field, (uv * field_size + offset) / field_size);
}

void main() {
    vec4 u = texture2D(field, frag_uv);
    // right
    vec4 u_1_i = textureOffset(frag_uv, vec2(-1, 0));
    // left
    vec4 u_i_1 = textureOffset(frag_uv, vec2(1, 0));

    // up
    vec4 u_j_1 = textureOffset(frag_uv, vec2(0, -1));
    // down
    vec4 u_1_j = textureOffset(frag_uv, vec2(0, 1));

    vec4 F = texture2D(external_force, frag_uv);

    // GAME LOOP ITERATION HERE

    float new_u = 0.0;

    gl_FragColor = pack(new_u);
} 