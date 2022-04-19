import * as sor from "sands-of-rust";
import ZingTouch from 'zingtouch';

const canvas = document.getElementById("sands-of-rust-canvas");
const brect = canvas.getBoundingClientRect();
canvas.setAttribute('width', brect.width);
canvas.setAttribute('height', brect.height);

let activeRegion = ZingTouch.Region(canvas);

const cellsPerInch = 50;
const ppi = window.devicePixelRatio * 96;
const mWidth = Math.floor(brect.width / ppi * cellsPerInch);
const mHeight = Math.floor(brect.height / ppi * cellsPerInch)
const fps = 120;

let r = sor.Render.new("sands-of-rust-canvas", mWidth, mHeight);

// fetch('https://picsum.photos/200/300').then(res => res.blob()).then(r => r.arrayBuffer()).then(r => blobToImage(r))

// const blobToImage = (blob) => {
//   return new Promise(resolve => {
//     // const url = URL.createObjectURL(blob)
//     // let img = new Image()
//     // img.onload = () => {
//     //   URL.revokeObjectURL(url)
//     //   resolve(img)
//     // }
//     // img.src = url
//     console.log('img', blob)
//   })
// }

let pan = new ZingTouch.Pan();

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

let selected = '5px solid rgb(255, 255, 255)';
let unselected = '1px solid rgb(255, 255, 255)';

Object.keys(sor.CellType).filter(isNaN).forEach(
  e =>  {
    var brushContainer = document.createElement("div");
    brushContainer.className = 'brush';

    var brush = document.createElement("div");
    brush.style.backgroundColor = sor.color_hex(sor.CellType[e]);
    brush.style.width = '80px';
    brush.style.height = '80px';
    brush.style.margin = '3px';
    brush.style.border = unselected;
    brushContainer.appendChild(brush);

    brushContainer.addEventListener('pointerup', () => {
      r.brush_change_color(sor.CellType[e]);
      Array.from(brushSelector.children)
        .filter(c => c.className == 'brush')
        .forEach(c => c.children[0].style.border = unselected)
      brush.style.border = selected
    })

    brushSelector.appendChild(brushContainer);
  }
);


let update = true;
let lastCall = 0;
let cum = 0;
let timeStep = 0;

let play_pause = document.getElementById('play-pause');

play_pause.addEventListener('pointerup', () => {
  update = !update;
  let pp = play_pause.getElementById('pp')
  if (update) {
    pp.setAttributeNS(null, 'points', '5,5 5,35 35,35 35,5');
  } else {
    pp.setAttributeNS(null, 'points', '5, 5 5,35 35 20');
  }
})

let recording = false;
let mediaRecorder;
let recordedChunks;

let record = document.getElementById('record');
record.addEventListener('pointerup', () => {
  recording = !recording;
  let cc = record.getElementById('cc')

  if (recording) {
    const stream = canvas.captureStream(25);
    mediaRecorder = new MediaRecorder(stream, { mimeType: 'video/webm;codecs=vp8' });
    recordedChunks = [];
    mediaRecorder.ondataavailable = e => {
      if (e.data.size > 0) {
        recordedChunks.push(e.data);
      }
    };
    mediaRecorder.start();

    cc.setAttributeNS(null, 'fill', 'yellow');
  } else {
    mediaRecorder.stop();
    setTimeout(() => {
      const blob = new Blob(recordedChunks, {
        type: "video/webm"
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "recording.webm";
      a.click();
      URL.revokeObjectURL(url);
    }, 300);

    cc.setAttributeNS(null, 'fill', 'red');
  }
})

const renderLoop = (timestamp) => {
  const delta = timestamp - lastCall;
  lastCall = timestamp;
  cum += delta;

  if (cum > 1000 / fps) {
    r.frame(timeStep, update);
    cum = 0;
    timeStep += 1;
  }

  requestAnimationFrame(renderLoop);
}

requestAnimationFrame(renderLoop);
