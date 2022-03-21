import * as sor from "sands-of-rust";

const canvas = document.getElementById("sands-of-rust-canvas");
const brect = canvas.getBoundingClientRect();
canvas.setAttribute('width', brect.width);
canvas.setAttribute('height', brect.height);

const cellsPerInch = 100;
const ppi = window.devicePixelRatio * 96;
const mWidth = Math.floor(brect.width / ppi * cellsPerInch);
const mHeight = Math.floor(brect.height / ppi * cellsPerInch)

let r = sor.Render.new("sands-of-rust-canvas", mWidth, mHeight);

let lastCall = 0;
let cum = 0;
let timeStep = 0;

const renderLoop = (timestamp) => {
  const delta = timestamp - lastCall;
  lastCall = timestamp;
  cum += delta;

  let fps = 60;
  if (cum > 1000 / fps) {
    r.frame(timeStep);
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
    // cursor offsets from top left
    // but gl coordinates offset from bottom left
    // can check coordinates in QUAD UVs
    const canvasTop = 1 - (ev.clientY - boundingRect.top) / boundingRect.height;

    r.brush_move_to(canvasLeft, canvasTop);
    r.brush_change_radius(10);
  }
});

canvas.addEventListener('pointerup', ev => {
  isDown = false;
  r.brush_change_radius(0);
});

// let color_picker = document.getElementById("brush-selector")
// for (let item in Object.keys(sor.CellType)) {
//   // color_picker.appendChild();
//   console.log("ite ", item);
//   console.log("v ", sor.CellType[item]);
//   console.log("bt ", item, sor.color_hex(item));
// }

document.addEventListener('keydown', ev => {
  console.log("Key down ", ev);
  if (ev.key == "1") {
    r.brush_change_color(sor.CellType.Sand)
  } else if (ev.key == "2") {
    r.brush_change_color(sor.CellType.Empty)
  } else if (ev.key == "3") {
    r.brush_change_color(sor.CellType.Wall)
  }
})

requestAnimationFrame(renderLoop);
