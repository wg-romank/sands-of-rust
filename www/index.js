import * as sor from "sands-of-rust";

const canvas = document.getElementById("sands-of-rust-canvas");
const brect = canvas.getBoundingClientRect();
const w = 64;
const h = 64;

canvas.setAttribute('width', brect.width);
canvas.setAttribute('height', brect.height);

const display_shader = sor.display_shader();
const compute_shader = sor.update_shader();
const copy_shader = sor.copy_shader();
const force_field = sor.Field.new(w, h);
const state = sor.initial_state(force_field, w, h);

let lastCall = 0;
let cum = 0;
let timeStep = 0;

const renderLoop = (timestamp) => {
  const delta = timestamp - lastCall;
  lastCall = timestamp;
  cum += delta;

  let fps = 10;
  if (cum > 1000 / fps) {
    sor.animation_frame(
        display_shader,
        compute_shader,
        copy_shader,
        force_field,
        state,
        timeStep
    );
    cum = 0;
    timeStep += 1;
  }

  requestAnimationFrame(renderLoop);
}

let isDown = false;
let button = 0;

canvas.addEventListener('pointerdown', ev => {
  isDown = true;
  button = ev.button;
});


canvas.addEventListener('pointermove', ev => {
  if (isDown) {
    const boundingRect = canvas.getBoundingClientRect();

    const canvasLeft = (ev.clientX - boundingRect.left) / boundingRect.width;
    const canvasTop = 1 - (ev.clientY - boundingRect.top) / boundingRect.height;

    let force = button == 0 ? 1000 : -10000;

    force_field.apply_force(canvasTop, canvasLeft, force, 5);
  }
});

canvas.addEventListener('pointerup', ev => {
  isDown = false;
});

requestAnimationFrame(renderLoop);
