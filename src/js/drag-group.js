import { px_var } from "./helpers.js";
import { snap_position } from "./multi-editor.js";

let state = {
  group: null,
  start_offset: null,
};

function finish() {
  state.group.classList.remove("dragging");
  let { x, y } = snap_position({
    x: px_var(state.group, "left"),
    y: px_var(state.group, "top"),
  });

  state.group.style.left = `${x}px`;
  state.group.style.top = `${y}px`;
  state = {
    group: null,
    start_offset: null,
  };
}

window.addEventListener("mousedown", (e) => {
  if (!e.target.matches(".--editor-group")) return;

  let group = e.target;

  let { left, top } = group.getBoundingClientRect();

  let x_offset = px_var(document.body, "--x-offset"),
    y_offset = px_var(document.body, "--y-offset");

  group.classList.add("dragging");

  state = {
    group,
    start_offset: {
      x: e.clientX - left + x_offset,
      y: e.clientY - top + y_offset,
    },
  };
});

window.addEventListener("mousemove", (e) => {
  if (!state.group) return;

  state.group.style.left = `${e.clientX - state.start_offset.x}px`;
  state.group.style.top = `${e.clientY - state.start_offset.y}px`;
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);
