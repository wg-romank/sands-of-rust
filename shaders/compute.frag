precision highp float;

uniform sampler2D field;
uniform vec2 field_size;
uniform float time_step;

uniform vec2 position;
uniform float color;
uniform float radius;

varying vec2 frag_uv;

vec4 textureOffset(vec2 uv, vec2 offset) {
  // todo: handle borders?
  // handling with clamp currently
  vec2 pt = position - uv;
  float radius_adjusted = radius / field_size.x;
  float color_norm = color / 255.;

  float img_component = texture2D(field, (uv * field_size + offset) / field_size).x;
  float force_component = clamp(sign(pow(radius_adjusted, 2.) - dot(pt, pt)), 0., 1.) * color_norm;

  if (force_component != 0.0) {
    return vec4(force_component, 0., 0., 0.);
  } else {
    return vec4(img_component, 0., 0., 0.);
  }
}

int gridIndex(vec2 coord) {
  vec2 inverted_coord = vec2(coord.x, 1.0 - coord.y);
  vec2 coord_scaled = inverted_coord * field_size;
  float x = floor(mod(coord_scaled.x, 2.));
  float y = floor(mod(coord_scaled.y, 2.));

  if (x == .0 && y == .0) {
    // * 2
    // 3 4
    return 1; // focus
  } else if (y == .0) {
    // 1 *
    // 3 4
    return 2; // right
  } else if (x == .0) {
    // 1 2
    // * 4
    return 3; // down
  } else {
    // 1 2
    // 3 *
    return 4; // down right
  }
}

// Reference: Cellular Gravity Tromp, Gruau
int timedGridIndex(vec2 coord, float time_step) {
  int idx = gridIndex(coord);
  int time_step_int = int(mod(time_step, 2.));

  if (time_step_int == 0) {
    return idx;
  } else if (time_step_int == 1) {
    // 1 -> 4
    // 2 -> 3
    // 3 -> 2
    // 4 -> 1
    if (idx == 1) {
      return 4;
    } else if (idx == 2) {
      return 3;
    } else if (idx == 3) {
      return 2;
    } else if (idx == 4) {
      return 1;
    }
  }
  return -1;
}

vec4 neighborhood(vec2 uv, int gid) {
  // time goes 0, 1, 0, 1, ...
  // need to apply mask based on own coordinates
  // instead of same pattern over whole picture
  // it must use particular parts with respect to current iteration (even grid, odd grid)

  vec2 offsetC1 = vec2(0, 0);
  vec2 offsetC2 = vec2(0, 0);
  vec2 offsetC3 = vec2(0, 0);
  vec2 offsetC4 = vec2(0, 0);

  // todo: time-based shifting
  if (gid == 1) { // focus == c1
    //  * 2
    //  3 4
    offsetC1 = vec2( 0, 0);
    offsetC2 = vec2(-1, 0); // right
    offsetC3 = vec2( 0, 1); // down
    offsetC4 = vec2(-1, 1); // down right
  } else if (gid == 2) { // right == c2
    //  1 *
    //  3 4
    offsetC1 = vec2( 1, 0); // left
    offsetC2 = vec2( 0, 0);
    offsetC3 = vec2( 1, 1); // down left
    offsetC4 = vec2( 0, 1); // down
  } else if (gid == 3) { // down == c3
    //  1 2
    //  * 3
    offsetC1 = vec2( 0,-1); // up
    offsetC2 = vec2(-1,-1); // up right
    offsetC3 = vec2( 0, 0);
    offsetC4 = vec2(-1, 0); // right
  } else if (gid == 4) { // down right == c4
    // 1 2
    // 3 *
    offsetC1 = vec2( 1,-1); // up left
    offsetC2 = vec2( 0,-1); // up 
    offsetC3 = vec2( 1, 0); // left
    offsetC4 = vec2( 0, 0);
  }

  // c1 c2
  // c3 c4
 
  vec4 c1 = textureOffset(uv, offsetC1);
  vec4 c2 = textureOffset(uv, offsetC2);
  vec4 c3 = textureOffset(uv, offsetC3);
  vec4 c4 = textureOffset(uv, offsetC4);

  return vec4(c1.x, c2.x, c3.x, c4.x);
}

vec4 codeNh(vec4 nh, int current) {
  return nh / 10.;
}

bool roughly_equals(vec3 nh, vec3 other) {
  vec3 result = abs(nh - other);

  return result.x < 0.1 && result.y < 0.1 && result.z < 0.1;
}

vec4 gravityBlackMagic(vec4 nh, int current) {
  // 1 2
  // 3 4
  
  // x y
  // z w

  // L liquid -> 2
  // * particle -> 1
  // ~ empty -> 0

  // particles (~) heavier than liquid (L)
  // empty space lighter than particles (*) and liquid (L)

  // for each iteration we assign heavy particle (*)
  // oil is ligher than water, it floats atop
  // oil is (L) while water is (*) and everyhing that is heavier than water is (*)

  // todo: walls?

  if (codeNh(nh, current) == vec4(2, 2, 2, 1)) {
    // L L  L L
    // L ~  ~ L
    return nh.xywz;
  } else if (codeNh(nh, current) == vec4(2, 2, 1, 2)) {
    // L L  L L
    // ~ L  L ~
    return nh.xywz;
  } else if (codeNh(nh, current) == vec4(2, 1, 3, 1)) {
    // L ~  ~ ~
    // * ~  * L
    return nh.wyzx;
  } else if (codeNh(nh, current) == vec4(1, 2, 1, 3)) {
    // ~ L  ~ ~
    // ~ *  L *
    return nh.xzyw;
  } else if (codeNh(nh, current) == vec4(2, 1, 3, 3)) {
    // L ~  ~ L
    // * *  * *
    return nh.yxzw;
  } else if (codeNh(nh, current) == vec4(1, 2, 3, 3)) {
    // ~ L  L ~
    // * *  * *
    return nh.yxzw;
  } else if (codeNh(nh, current) == vec4(1, 1, 2, 1)) {
    // ~ ~  ~ ~
    // L ~  ~ L
    return nh.xywz;
  } else if (codeNh(nh, current) == vec4(1, 1, 1, 2)) {
    // ~ ~  ~ ~
    // ~ L  L ~
    return nh.xywz;
  } else if (roughly_equals(codeNh(nh, current).yzw, vec3(1, 1, 1))) {
    // * ~  ~ ~
    // ~ ~  * ~
    return nh.zyxw;
  } else if (codeNh(nh, current).xzw == vec3(1, 1, 1)) {
    // ~ *  ~ ~
    // ~ ~  ~ *
    return nh.xwzy;
  } else if (codeNh(nh, current).xz == vec2(1, 1)) {
    // ~ *  ~ ~
    // ~ *  * *
    return nh.xzyw;
  } else if (codeNh(nh, current).zw == vec2(1, 1)) {
    // * *  ~ ~
    // ~ ~  * *
    return nh.zwxy;
  } else if (codeNh(nh, current).xw == vec2(1, 1)) {
    // ~ *  ~ ~
    // * ~  * *
    return nh.xwzy;
  } else if (codeNh(nh, current).yz == vec2(1, 1)) {
    // * ~  ~ ~
    // ~ *  * *
    return nh.zyxw;
  } else if (codeNh(nh, current).yw == vec2(1, 1)) {
    // * ~  ~ ~
    // * ~  * *
    return nh.wyzx;
  } else if (codeNh(nh, current).z == 1.) {
    // * *  ~ *
    // ~ *  * *
    return nh.zyxw;
  } else if (codeNh(nh, current).w == 1.) {
    // * *  * ~
    // * ~  * *
    return nh.xwzy;
  } else {
    return nh;
  }
}

vec4 vectorId(int gid) {
  if (gid == 1) {
    return vec4(1, 0, 0, 0);
  } else if (gid == 2) {
    return vec4(0, 1, 0, 0);
  } else if (gid == 3) {
    return vec4(0, 0, 1, 0);
  } else {
    return vec4(0, 0, 0, 1);
  }
}

vec4 decodeNeighborhood(int gid, vec4 nh) {
  float s = dot(vectorId(gid), nh);

  return vec4(s, 0, 0, 0);
}

void main() {
  int gid = timedGridIndex(frag_uv, time_step);

  vec4 mask = neighborhood(frag_uv, gid);

  // values passed in texture are scaled from 0 to 1
  // for (int i = 0; i < 10; i = i + 1) {
  vec4 shiftedMask = gravityBlackMagic(mask * 255., 0) / 255.;
  // }

  gl_FragColor = decodeNeighborhood(gid, shiftedMask);
} 
