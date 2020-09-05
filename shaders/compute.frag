precision highp float;

uniform sampler2D field;
uniform sampler2D external_force;
uniform vec2 field_size;
uniform float time_step;

varying vec2 frag_uv;

vec4 textureOffset(vec2 uv, vec2 offset) {
  // todo: handle borders?
  // handling with clamp currently
  return texture2D(field, (uv * field_size + offset) / field_size);
}

vec4 forceOffset(vec2 uv, vec2 offset) {
  // todo: handle borders?
  // handling with clamp currently
  return texture2D(external_force, (uv * field_size + offset) / field_size);
}

const int EMPTY = 0x0;
const int SAND = 0x1;

int encodeCell(vec4 contents) {
  if (contents.x > 0.) {
      return SAND;
  } else {
      return EMPTY;
  }
}

vec4 decodeCell(int value) {
  if (value == SAND) {
    return vec4(1., 0., 0., 1.);
  } else {
    return vec4(0., 0., 0., 1.);
  }
}

vec2 timeOffset(float time_step) {
  if (mod(time_step, 4.) == 0.) {
    return vec2(0, 0);
  } else if (mod(time_step, 4.) == 1.) {
    return vec2(-1, 0); // right
  } else if (mod(time_step, 4.) == 2.) {
    return vec2(0, 1); // down
  } else { // time_step % 4 == 3
    return vec2(-1, 1); // down, right
  }
}

int encode(vec4 contents, int position) {
  if (contents.x > 0.) {
    if (position == 0) {
      return 0x1000;
    } else if (position == 1) {
      return 0x0100;
    } else if (position == 2) {
      return 0x0010;
    } else {
      return 0x0001;
    }
  }
}

int neighborhood(vec2 uv, float time_step) {
  // time goes 0, 1, 2, 3, 0, 1, ...
  // need to apply mask based on own coordinates
  // instead of same pattern over whole picture
  // it must use particular parts with respect to current iteration (zero shift, shift x, shift y)
  vec2 offsetFocus = vec2(0, 0) + timeOffset(time_step);
  vec2 offsetRight = vec2(-1, 0) + timeOffset(time_step);
  vec2 offsetDown = vec2(0, 1) + timeOffset(time_step);
  vec2 offsetDownRight = vec2(-1, 1) + timeOffset(time_step);

  // vec4 c1 = textureOffset(uv, vec2( 0, 0)) + forceOffset(uv, vec2( 0, 0));
  // vec4 c2 = textureOffset(uv, vec2(-1, 0)) + forceOffset(uv, vec2(-1, 0)); // right
  // vec4 c3 = textureOffset(uv, vec2( 0, 1)) + forceOffset(uv, vec2( 0, 1)); // down
  // vec4 c4 = textureOffset(uv, vec2(-1, 1)) + forceOffset(uv, vec2(-1, 1)); // righ + down

  vec4 c1 = textureOffset(uv, offsetFocus);
  vec4 c2 = textureOffset(uv, offsetRight);
  vec4 c3 = textureOffset(uv, offsetDown);
  vec4 c4 = textureOffset(uv, offsetDownRight);

  // bigendinan?
  // s1 -> 0x1000
  // s2 -> 0x0100
  // s3 -> 0x0010
  // s4 -> 0x0001
  int s1 = encodeCell(c1);
  int s2 = encodeCell(c2) * 2;
  int s3 = encodeCell(c3) * 2 * 2;
  int s4 = encodeCell(c4) * 2 * 2 * 2;

  return s1 + s2 + s3 + s4;
}

void main() {
  int mask = neighborhood(frag_uv, time_step);

  // int next_value = 0;
  // if (mask == 0x0001) {
  //   next_value = 0x0010;
  // } else {
  //   next_value = mask;
  // }

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

  gl_FragColor = decodeCell(mask);
} 