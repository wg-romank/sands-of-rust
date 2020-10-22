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
const state = sor.initial_state(w, h);

let lastCall = 0;
let cum = 0;
let timeStep = 0;

let x = 0
let y = 0;
let color = sor.CellType.Water;
let radius = 0;

const renderLoop = (timestamp) => {
  const delta = timestamp - lastCall;
  lastCall = timestamp;
  cum += delta;

  let fps = 20;
  if (cum > 1000 / fps) {
    sor.animation_frame(
        display_shader,
        compute_shader,
        copy_shader,
        x,
        y,
        color,
        radius,
        force_field,
        state,
        timeStep
    );
    cum = 0;
    timeStep += 1;

    radius = 0;
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
    const canvasTop = (ev.clientY - boundingRect.top) / boundingRect.height;

    x = canvasLeft;
    y = canvasTop;
    radius = 2;
  }
});

canvas.addEventListener('pointerup', ev => {
  isDown = false;
});

canvas.addEventListener('pointerleave', ev => {
  if (color == sor.CellType.Water) {
    color = sor.CellType.Sand;
  } else {
    color = sor.CellType.Water;
  }
});

requestAnimationFrame(renderLoop);
