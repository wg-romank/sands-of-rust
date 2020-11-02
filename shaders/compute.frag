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

  float cutoff = max(img_component, force_component);

  return vec4(clamp(img_component + force_component, 0., cutoff), 0., 0., 0.);
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

  // vec4 c1 = textureOffset(uv, vec2( 0, 0)) + forceOffset(uv, vec2( 0, 0));
  // vec4 c2 = textureOffset(uv, vec2(-1, 0)) + forceOffset(uv, vec2(-1, 0)); // right
  // vec4 c3 = textureOffset(uv, vec2( 0, 1)) + forceOffset(uv, vec2( 0, 1)); // down
  // vec4 c4 = textureOffset(uv, vec2(-1, 1)) + forceOffset(uv, vec2(-1, 1)); // righ + down


  // c1 c2
  // c3 c4
 
  vec4 c1 = textureOffset(uv, offsetC1);
  vec4 c2 = textureOffset(uv, offsetC2);
  vec4 c3 = textureOffset(uv, offsetC3);
  vec4 c4 = textureOffset(uv, offsetC4);

  return vec4(c1.x, c2.x, c3.x, c4.x);
}

vec4 gravityBlackMagic(vec4 nh) {
  // 1 2
  // 3 4
  
  // x y
  // z w

  if (nh == vec4(2, 2, 2, 0)) {
    // L L  L L
    // L ~  ~ L
    return vec4(2, 2, 0, 2);
  } else if (nh == vec4(2, 0, 1, 1)) {
    // L ~  ~ L
    // * *  * *
    return vec4(0, 2, 1, 1);
  } else if (nh == vec4(0, 2, 1, 1)) {
    // ~ L  L ~
    // * *  * *
    return vec4(2, 0, 1, 1);
  } else if (nh == vec4(0, 0, 2, 0)) {
    // ~ ~  ~ ~
    // L ~  ~ L
    return vec4(0, 0, 0, 2);
  } else if (nh == vec4(0, 0, 0, 2)) {
    // ~ ~  ~ ~
    // ~ L  L ~
    return vec4(0, 0, 2, 0);
  } else if (nh.yzw == vec3(0, 0, 0)) {
    // * ~  ~ ~
    // ~ ~  * ~
    return nh.zyxw;
  } else if (nh.xzw == vec3(0, 0, 0)) {
    // ~ *  ~ ~
    // ~ ~  ~ *
    return nh.xwzy;
  } else if (nh.xz == vec2(0, 0)) {
    // ~ *  ~ ~
    // ~ *  * *
    return nh.xzyw;
  } else if (nh.zw == vec2(0, 0)) {
    // * *  ~ ~
    // ~ ~  * *
    return nh.zwxy;
  } else if (nh.xw == vec2(0, 0)) {
    // ~ *  ~ ~
    // * ~  * *
    return nh.xwzy;
  } else if (nh.yz == vec2(0, 0)) {
    // * ~  ~ ~
    // ~ *  * *
    return nh.zyxw;
  } else if (nh.yw == vec2(0, 0)) {
    // * ~  ~ ~
    // * ~  * *
    return nh.wyzx;
  } else if (nh.z == 0.) {
    // * *  ~ *
    // ~ *  * *
    return nh.zyxw;
  } else if (nh.w == 0.) {
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

  vec4 shiftedMask = gravityBlackMagic(mask * 255.) / 255.;

  // if (gid == 1) {
  //   gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
  // } else if (gid == 2) {
  //   gl_FragColor = vec4(0.5, 0, 0.5, 1.0);
  // } else if (gid == 3) {
  //   gl_FragColor = vec4(0.5, 1.0, 0.5, 1.0);
  // } else if (gid == 4) {
  //   gl_FragColor = vec4(0.5, 0.5, 1.0, 1.0);
  // }


  gl_FragColor = decodeNeighborhood(gid, shiftedMask);
} 