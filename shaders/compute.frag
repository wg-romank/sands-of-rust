precision highp float;

uniform sampler2D field;
uniform vec2 field_size;
uniform float time_step;

uniform vec2 position;
uniform float color;
uniform float radius;
uniform float sand_speed;

varying vec2 frag_uv;

vec4 textureOffset(vec2 uv, vec2 offset) {
  return texture2D(field, (uv * field_size + offset) / field_size);
}

float brush(vec2 uv) {
  vec2 pt = position - uv; // adjusted for center of brush
  float radius_adjusted = radius / field_size.x;
  float color_norm = color / 255.;

  float in_circle = sign(pow(radius_adjusted, 2.) - dot(pt, pt));
  float force_component = clamp(in_circle, 0., 1.) * color_norm;

  return force_component;
}

vec4 texture(vec2 uv, vec2 offset) {
  float force_component = brush(uv);
  if (force_component != 0.0) {
    return vec4(force_component, 0., 0., 0.);
  } else {
    return vec4(textureOffset(uv, offset).x, 0., 0., 0.);
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
    offsetC3 = vec2( 0,-1); // down
    offsetC4 = vec2(-1,-1); // down right
  } else if (gid == 2) { // right == c2
    //  1 *
    //  3 4
    offsetC1 = vec2( 1, 0); // left
    offsetC2 = vec2( 0, 0);
    offsetC3 = vec2( 1,-1); // down left
    offsetC4 = vec2( 0,-1); // down
  } else if (gid == 3) { // down == c3
    //  1 2
    //  * 3
    offsetC1 = vec2( 0, 1); // up
    offsetC2 = vec2(-1, 1); // up right
    offsetC3 = vec2( 0, 0);
    offsetC4 = vec2(-1, 0); // right
  } else if (gid == 4) { // down right == c4
    // 1 2
    // 3 *
    offsetC1 = vec2( 1, 1); // up left
    offsetC2 = vec2( 0, 1); // up
    offsetC3 = vec2( 1, 0); // left
    offsetC4 = vec2( 0, 0);
  }

  // c1 c2
  // c3 c4
 
  vec4 c1 = texture(uv, offsetC1);
  vec4 c2 = texture(uv, offsetC2);
  vec4 c3 = texture(uv, offsetC3);
  vec4 c4 = texture(uv, offsetC4);

  return vec4(c1.x, c2.x, c3.x, c4.x);
}

vec4 gravityBlackMagic(vec4 nh) {
  // 1 2
  // 3 4
  
  // x y
  // z w

  // * particle -> > 0
  // ~ empty -> 0

  // empty space lighter than particles (*)
  float ss = sand_speed / 255.;

  if (nh.x > nh.z && nh.yw == vec2(0., 0.) && nh.z + ss < 1.) {
    // * ~  ~ ~
    // ~ ~  * ~
    return vec4(nh.x - ss, nh.y, nh.z + ss, nh.w);
  } else if (nh.y > nh.w && nh.xz == vec2(0., 0.) && nh.w + ss < 1.) {
    // ~ *  ~ ~
    // ~ ~  ~ *
    return vec4(nh.x, nh.y - ss, nh.z, nh.w + ss);
  } else if (nh.y > 0. && nh.z > nh.w && nh.x == 0. && nh.w + ss < 1.) {
    // ~ *  ~ ~
    // * ~  * *
    return vec4(nh.x, nh.y - ss, nh.z, nh.w + ss);
  } else if (nh.x > 0. && nh.w > nh.z && nh.y == 0. && nh.z + ss < 1.) {
    // * ~  ~ ~
    // ~ *  * *
    return vec4(nh.x - ss, nh.y, nh.z + ss, nh.w);
  } else if (nh.y > 0. && nh.z < nh.w && nh.x == 0. && nh.z + ss < 1.) {
    // ~ *  ~ ~
    // ~ *  * *
    return vec4(nh.x, nh.y - ss, nh.z + ss, nh.w);
  } else if (nh.x > 0. && nh.z > nh.w && nh.y == 0. && nh.w + ss < 1.) {
    // * ~  ~ ~
    // * ~  * *
    return vec4(nh.x - ss, nh.y, nh.z, nh.w + ss);
  } else if (nh.x > 0. && nh.y > 0. && nh.w > nh.z && nh.z + ss < 1.) {
    // * *  ~ *
    // ~ *  * *
    return vec4(nh.x - ss, nh.y, nh.z + ss, nh.w);
  } else if (nh.x > 0. && nh.y > 0. && nh.z > nh.w && nh.w + ss < 1.) {
    // * *  * ~
    // * ~  * *
    return vec4(nh.x, nh.y - ss, nh.z, nh.w + ss);
  } else if (nh.x > 0. && nh.y > 0. && nh.x > nh.z && nh.y > nh.w && nh.w + ss < 1. && nh.z + ss < 1.) {
    // * *  ~ ~
    // ~ ~  * *
    return vec4(nh.x - ss, nh.y - ss, nh.z + ss, nh.w + ss);
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
  vec4 shiftedMask = gravityBlackMagic(mask);
  // }

  gl_FragColor = decodeNeighborhood(gid, shiftedMask);
} 
