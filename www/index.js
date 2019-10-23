import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 6;
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
const universeSpeedRange = document.getElementById("universe-speed");
const universeSpeedText = document.getElementById("universe-speed-num");
const canvas = document.getElementById("game-of-life-canvas");
canvas.width = (CELL_SIZE + 1) * width + 1;
canvas.height = (CELL_SIZE + 1) * height + 1;

const ctx = canvas.getContext("2d");

const universeSpeed = () => {
  return 1001 - parseInt(universeSpeedRange.value, 10);
};

universeSpeedRange.addEventListener("change", event => {
  universeSpeedText.textContent = `${universeSpeedRange.value / 10}%`;
  drawGrid();
  drawCells();
});

const renderLoop = async () => {
  // debugger;
  animationSpeedTimeout = await setTimeout(() => {
    universe.tick();
    if (animationId) {
      stepCount.textContent = Math.ceil(animationId / 2);
    }
    drawGrid();
    drawCells();

    animationId = requestAnimationFrame(renderLoop);
  }, universeSpeed());
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
    isPaused() ? play() : pause()
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

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      ctx.fillStyle = bitIsSet(idx, cells) ? ALIVE_COLOR : DEAD_COLOR;
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

document.addEventListener("onkeydown", e => {
  if (e.code == "Space") {
    isPaused() ? play() : pause()
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
