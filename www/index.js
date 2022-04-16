import * as sor from "sands-of-rust";
import ZingTouch from 'zingtouch';

const canvas = document.getElementById("sands-of-rust-canvas");
const brect = canvas.getBoundingClientRect();
canvas.setAttribute('width', brect.width);
canvas.setAttribute('height', brect.height);

let activeRegion = ZingTouch.Region(canvas);

const cellsPerInch = 100;
const ppi = window.devicePixelRatio * 96;
const mWidth = Math.floor(brect.width / ppi * cellsPerInch);
const mHeight = Math.floor(brect.height / ppi * cellsPerInch)

let r = sor.Render.new("sands-of-rust-canvas", mWidth, mHeight);

let pan = new ZingTouch.Pan();

let update = true;

activeRegion.bind(canvas, pan, e => {
  let ev = e.detail.events[0];
  const boundingRect = canvas.getBoundingClientRect();

  const canvasLeft = (ev.clientX - boundingRect.left) / boundingRect.width;
  // cursor offsets from top left
  // but gl coordinates offset from bottom left
  // can check coordinates in QUAD UVs
  const canvasTop = 1 - (ev.clientY - boundingRect.top) / boundingRect.height;

  r.brush_change_radius(10);
  r.brush_move_to(canvasLeft, canvasTop);
})

pan.end = () => { r.brush_change_radius(0) }

let brushSelector = document.getElementById('brush-selector');

Object.keys(sor.CellType).filter(isNaN).forEach(
  e =>  {
    var brushContainer = document.createElement("div");
    brushContainer.className = 'brush';

    var brush = document.createElement("div");
    brush.style.backgroundColor = sor.color_hex(sor.CellType[e]);
    brush.style.width = '80px';
    brush.style.height = '80px';
    brush.style.margin = '3px';
    brush.style.border = '1px solid rgb(255, 255, 255)';
    brushContainer.appendChild(brush);

    brushContainer.addEventListener('pointerup', () => {
      r.brush_change_color(sor.CellType[e]);
      console.log("EE")
    })

    brushSelector.appendChild(brushContainer);
  }
);

let lastCall = 0;
let cum = 0;
let timeStep = 0;

const renderLoop = (timestamp) => {
  const delta = timestamp - lastCall;
  lastCall = timestamp;
  cum += delta;

  let fps = 60;
  if (cum > 1000 / fps) {
    r.frame(timeStep, update);
    cum = 0;
    timeStep += 1;
  }

  requestAnimationFrame(renderLoop);
}

requestAnimationFrame(renderLoop);
