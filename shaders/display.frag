precision highp float;

uniform sampler2D field;
uniform vec2 force_dir;

varying vec2 frag_uv;

float unpack(vec4 packedZValue) {
    const vec4 bitShifts = vec4(1.0 / (256.0 * 256.0 * 256.0),
                        1.0 / (256.0 * 256.0),
                        1.0 / 256.0,
                        1);
    float shadow = dot(packedZValue , bitShifts);
    return shadow;
}

void main() {
    float packedZValue = unpack(texture2D(field, frag_uv - force_dir));

    // todo: proper color conversion? (estimate field value u limits)
    float mValue = clamp(packedZValue, 0.0, 1.0);
    gl_FragColor = vec4(mValue, mValue, mValue, 1.0);
}