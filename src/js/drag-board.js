import { px_var } from "./helpers.js";

let is_dragging = false;
let state = {
  start_offset: null,
  start_mouse_position: null,
};

function finish() {
  is_dragging = false;
  state = {
    start_offset: null,
    start_mouse_position: null,
  };
  document.body.classList.remove("panning");
}

window.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  if (e.target.classList.contains("canvas")) {
    document.body.classList.add("panning");

    is_dragging = true;
    state = {
      start_offset: {
        x: px_var(document.body, "--x-offset"),
        y: px_var(document.body, "--y-offset"),
      },
      start_mouse_position: {
        x: e.clientX,
        y: e.clientY,
      },
    };
  }
});

window.addEventListener("mousemove", (e) => {
  if (is_dragging) {
    document.body.style.setProperty(
      "--x-offset",
      `${e.clientX - state.start_mouse_position.x + state.start_offset.x}px`,
    );
    document.body.style.setProperty(
      "--y-offset",
      `${e.clientY - state.start_mouse_position.y + state.start_offset.y}px`,
    );
  }
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);
window.addEventListener("mouseleave", finish);
window.addEventListener("keydown", finish);
