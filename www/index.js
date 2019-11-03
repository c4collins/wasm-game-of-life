import { Universe } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 1;
const GRID_COLOR = "#cccccc";
const DEAD_COLOR = "#ffffff";
const ALIVE_COLOR = "#000000";

const universe = Universe.new();
const width = universe.width();
const height = universe.height();
let animationId = null;
let animationSpeedTimeout = null;

const playPauseButton = document.getElementById("play-pause");
const stepCount = document.getElementById("step-count");
const clearButton = document.getElementById("empty-universe");
const randomButton = document.getElementById("random-universe");
const canvas = document.getElementById("game-of-life-canvas");
canvas.width = (CELL_SIZE + 1) * width + 1;
canvas.height = (CELL_SIZE + 1) * height + 1;

const ctx = canvas.getContext("2d");

const renderLoop = async () => {
  // debugger;
    fps.render();
    universe.tick();
    if (animationId) stepCount.textContent = Math.ceil(animationId / 2);
    drawGrid();
    drawCells();

    animationId = requestAnimationFrame(renderLoop);
};

const isPaused = () => {
  return animationId === null;
};

const play = () => {
  console.log("playing");
  playPauseButton.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  console.log("pausing");
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  clearTimeout(animationSpeedTimeout);
  animationId = null;
};

playPauseButton.addEventListener("click", () =>
  isPaused() ? play() : pause()
);

document.addEventListener("keydown", e => {
  if (e.code == "KeyP") {
    isPaused() ? play() : pause();
  }
  //
});

randomButton.addEventListener("click", () => {
  pause();
  universe.randomize_cells();
  drawGrid();
  drawCells();
});
clearButton.addEventListener("click", () => {
  pause();
  universe.clear_cells();
  drawGrid();
  drawCells();
});

const getIndex = (row, column) => {
  return row * width + column;
};

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // vertical lines
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }
  // horizontal lines
  for (let j = 0; j <= width; j++) {
    ctx.moveTo(0, j * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const bitIsSet = (n, arr) => {
  const byte = Math.floor(n / 8);
  const mask = 1 << n % 8;
  return (arr[byte] & mask) === mask;
};

const drawCells = () => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, (width * height) / 8);

  ctx.beginPath();
  // fillStyle is expensive so set it as few times as possible
  ctx.fillStyle = ALIVE_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (!bitIsSet(idx, cells)) continue;
      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }
  ctx.fillStyle = DEAD_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (bitIsSet(idx, cells)) continue;
      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};

const fps = new (class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }
  render() {
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = (1 / delta) * 1000;

    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0, j = this.frames.length; i < j; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    this.fps.innerHTML = `<table>
      <tr><th colspan="2">Frames per Second:</th></tr>
      <tr><td>latest</td><td>${Math.round(fps)}</td></tr>
      <tr><td>avg of last 100</td><td>${Math.round(mean)}</td></tr>
      <tr><td>min of last 100</td><td>${Math.round(min)}</td></tr>
      <tr><td>max of last 100</td><td>${Math.round(max)}</td></tr>
    </table>`.trim();
  }
})();

document.addEventListener("onkeydown", e => {
  if (e.code == "Space") {
    isPaused() ? play() : pause();
  }
});

canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = event.clientX - boundingRect.left + scaleX;
  const canvasTop = event.clientY - boundingRect.top + scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  if (event.ctrlKey) {
    universe.create_object("glider", row, col);
  } else if (event.shiftKey) {
    universe.create_object("pulsar", row, col);
  } else {
    universe.toggle_cell(row, col);
  }

  drawGrid();
  drawCells();
});

drawGrid();
drawCells();
play();
