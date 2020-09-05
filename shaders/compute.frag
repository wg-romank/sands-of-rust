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

// todo: double-check logic here
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

vec4 encode(vec4 contents, int position) {
  if (contents.x > 0.) {
    if (position == 0) {
      return vec4(1, 0, 0, 0);
    } else if (position == 1) {
      return vec4(0, 1, 0, 0);
    } else if (position == 2) {
      return vec4(0, 0, 1, 0);
    } else if (position == 3) {
      return vec4(0, 0, 0, 1);
    } else {
      return vec4(0, 0, 0, 0);
    }
  } else {
    return vec4(0, 0, 0, 0);
  }
}

int gridIndex(vec2 coord) {
  float x = mod(coord.x, 2.);
  float y = mod(coord.y, 2.);

  if (x == .0 && y == .0) {
    // * 2
    // 3 4
    return 0; // focus
  } else if (y == .0) {
    // 1 *
    // 3 4
    return 1; // right
  } else if (x == .0) {
    // 1 2
    // * 4
    return 2; // down
  } else {
    // 1 2
    // 3 *
    return 3; // down right
  }
}

vec4 vectorId(vec2 coord) {
  int gid = gridIndex(coord);

  if (gid == 0) {
    return vec4(1, 0, 0, 0);
  } else if (gid == 1) {
    return vec4(0, 1, 0, 0);
  } else if (gid == 2) {
    return vec4(0, 0, 1, 0);
  } else {
    return vec4(0, 0, 0, 1);
  }
}

vec4 neighborhood(vec2 uv, float time_step) {
  int gridIndex = gridIndex(uv * field_size);

  // time goes 0, 1, 2, 3, 0, 1, ...
  // need to apply mask based on own coordinates
  // instead of same pattern over whole picture
  // it must use particular parts with respect to current iteration (zero shift, shift x, shift y)

  vec2 offsetC1 = vec2(0, 0);
  vec2 offsetC2 = vec2(0, 0);
  vec2 offsetC3 = vec2(0, 0);
  vec2 offsetC4 = vec2(0, 0);

  // todo: time-based shifting
  if (gridIndex == 0) { // focus == c1
    //  * 1
    //  2 3
    offsetC1 = vec2( 0, 0);
    offsetC2 = vec2(-1, 0); // right
    offsetC3 = vec2( 0, 1); // down
    offsetC4 = vec2(-1, 1); // down right
  } else if (gridIndex == 1) { // right == c2
    //  1 *
    //  2 3
    offsetC1 = vec2( 1, 0); // left
    offsetC2 = vec2( 0, 0);
    offsetC3 = vec2( 1, 1); // down left
    offsetC4 = vec2( 0, 1); // down
  } else if (gridIndex == 2) { // down == c3
    //  1 2
    //  * 3
    offsetC1 = vec2( 0,-1); // up
    offsetC2 = vec2(-1,-1); // up right
    offsetC3 = vec2( 0, 0);
    offsetC4 = vec2(-1, 0); // right
  } else if (gridIndex == 3) { // down right == c4
    // 1 2
    // 3 *
    offsetC1 = vec2( 1,-1); // up left
    offsetC2 = vec2( 0,-1); // up 
    offsetC3 = vec2( 1, 0); // left
    offsetC4 = vec2( 0, 0);
  }

  // vec4 c1 = textureOffset(uv, vec2( 0, 0)) + forceOffset(uv, vec2( 0, 0));
  // vec4 c2 = textureOffset(uv, vec2(-1, 0)) + forceOffset(uv, vec2(-1, 0)); // right
  // vec4 c3 = textureOffset(uv, vec2( 0, 1)) + forceOffset(uv, vec2( 0, 1)); // down
  // vec4 c4 = textureOffset(uv, vec2(-1, 1)) + forceOffset(uv, vec2(-1, 1)); // righ + down


  // c1 c2
  // c3 c4
 
  vec4 c1 = textureOffset(uv, offsetC1 + timeOffset(time_step));
  vec4 c2 = textureOffset(uv, offsetC2 + timeOffset(time_step));
  vec4 c3 = textureOffset(uv, offsetC3 + timeOffset(time_step));
  vec4 c4 = textureOffset(uv, offsetC4 + timeOffset(time_step));

  return encode(c1, 0) + encode(c2, 1) + encode(c3, 2) + encode(c4, 3);
}

vec4 gravityBlackMagic(vec4 nh) {
  // 1 2
  // 3 4

  if (nh == vec4(1, 0, 0, 0)) {
    // * ~  ~ ~
    // ~ ~  * ~
    return vec4(0, 0, 1, 0);
  } else if (nh == vec4(1, 1, 1, 0)) {
    // * *  * ~
    // * ~  * *
    return vec4(1, 0, 1, 1);
  } else if (nh == vec4(1, 1, 0, 0)) {
    // * *  ~ ~
    // ~ ~  * *
    return vec4(0, 0, 1, 1);
  } else if (nh == vec4(0, 1, 1, 0)) {
    // ~ *  ~ ~
    // * ~  * *
    return vec4(0, 0, 1, 1);
  } else if (nh == vec4(0, 1, 0, 0)) {
    // ~ *  ~ ~
    // ~ ~  ~ *
    return vec4(0, 0, 0, 1);
  } else if (nh == vec4(1, 1, 0, 1)) {
    // * *  ~ *
    // ~ *  * *
    return vec4(0, 1, 1, 1);
  } else if (nh == vec4(1, 0, 0, 1)) {
    // * ~  ~ ~
    // ~ *  * *
    return vec4(0, 0, 1, 1);
  } else if (nh == vec4(1, 0, 1, 0)) {
    // * ~  ~ ~
    // * ~  * *
    return vec4(0, 0, 1, 1);
  } else if (nh == vec4(0, 1, 0, 1)) {
    // ~ *  ~ ~
    // ~ *  * *
    return vec4(0, 0, 1, 1);
  } else {
    return nh;
  }
}

vec4 decodeNeighborhood(vec2 uv, vec4 nh) {
  float s = dot(vectorId(uv), nh);

  if (s > 0.0) {
    return decodeCell(SAND);
  } else {
    return decodeCell(EMPTY);
  }
}

void main() {
  vec4 mask = neighborhood(frag_uv, time_step);

  vec4 shiftedMask = gravityBlackMagic(mask);

  gl_FragColor = decodeNeighborhood(frag_uv, shiftedMask);
} 