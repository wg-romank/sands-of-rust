precision highp float;

uniform sampler2D field;
uniform vec2 field_size;
uniform float time_step;

uniform sampler2D patterns;
uniform sampler2D rules;

uniform float num_rules;
uniform float rules_texture_size;
uniform float update;

uniform vec2 position;
uniform vec4 color;
uniform float radius;

varying vec2 frag_uv;

// todo: this needs to be adjusted if we have more rules
const float MAX_RULES_TEXTURE_SIZE = 512.;

vec4 textureOffset(vec2 uv, vec2 offset) {
  return texture2D(field, (uv * field_size + offset) / field_size);
}

vec4 brush(vec2 uv) {
  vec2 pt = position - uv; // adjusted for center of brush
  float radius_adjusted = radius / field_size.x;

  float in_circle = sign(pow(radius_adjusted, 2.) - dot(pt, pt));
  vec4 force_component = clamp(in_circle, 0., 1.) * color;

  return force_component;
}

vec4 texture(vec2 uv, vec2 offset) {
  vec4 force_component = brush(uv);
  if (force_component != vec4(0)) {
    return force_component;
  } else {
    return textureOffset(uv, offset);
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
  int time_step_int = int(mod(time_step, 4.));

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
  } else if (time_step_int == 2) {
    // 1 -> 3
    // 2 -> 4
    // 3 -> 1
    // 4 -> 2
    if (idx == 1) {
      return 3;
    } else if (idx == 2) {
      return 4;
    } else if (idx == 3) {
      return 1;
    } else if (idx == 4) {
      return 2;
    }
  } else if (time_step_int == 3) {
    // 1 -> 2
    // 2 -> 1
    // 3 -> 4
    // 4 -> 3
    if (idx == 1) {
      return 2;
    } else if (idx == 2) {
      return 1;
    } else if (idx == 3) {
      return 4;
    } else if (idx == 4) {
      return 3;
    }
  }
  return -1;
}

mat4 neighborhood(vec2 uv, int gid) {
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

  return mat4(c1, c2, c3, c4);
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

vec4 gravityBlackMagic(mat4 nh, int gid) {
  // 1 2
  // 3 4
  // unable to use uniform in loop comparison

  for (float i = 0.; i < MAX_RULES_TEXTURE_SIZE; i += 2.) {
    if (i >= 2. * num_rules) {
      break;
    }
    // https://stackoverflow.com/a/60620232
    vec2 off1 = (vec2(0.5, 0.5) + vec2(i, 0)) / vec2(rules_texture_size, 2);
    vec2 off2 = (vec2(1.5, 0.5) + vec2(i, 0)) / vec2(rules_texture_size, 2);
    vec2 off3 = (vec2(0.5, 1.5) + vec2(i, 0)) / vec2(rules_texture_size, 2);
    vec2 off4 = (vec2(1.5, 1.5) + vec2(i, 0)) / vec2(rules_texture_size, 2);
  
    mat4 pattern = mat4(
      texture2D(patterns, off1),
      texture2D(patterns, off2),
      texture2D(patterns, off3),
      texture2D(patterns, off4)
    );

    if (pattern == nh) {
      if (gid == 1) {
        return texture2D(rules, off1);
      } else if (gid == 2) {
        return texture2D(rules, off2);
      } else if (gid == 3) {
        return texture2D(rules, off3);
      } else if (gid == 4) {
        return texture2D(rules, off4);
      }
    }
  }

  // todo: check
  return nh * vectorId(gid);
}

void main() {
  int gid = timedGridIndex(frag_uv, time_step);

  mat4 nh = neighborhood(frag_uv, gid);

  // values passed in texture are scaled from 0 to 1
  // for (int i = 0; i < 10; i = i + 1) {
  // gl_FragColor = nh * vectorId(gid);
  if (update != 0.0) {
    gl_FragColor = gravityBlackMagic(nh, gid);
  } else {
    gl_FragColor = nh * vectorId(gid);
  }
  // }
} 
