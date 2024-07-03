let current_editor = null;
let start_x, start_y;
let x_offset, y_offset;

let set_editor_position = (x, y) => {
  state.current_editor.style.left = `${x}px`;
  state.current_editor.style.top = `${y}px`;
  state.current_editor.dispatchEvent(new Event("moved"));
};

function left(elem) {
  return Number(elem.style.left.split("px")[0]);
}

function top(editor) {
  return Number(editor.style.top.split("px")[0]);
}

const SNAP_OFFSET = 4;
window.snap_editor = function (editor) {
  let x = left(editor);
  let y = top(editor);

  if (x % 25 < 9) editor.style.left = `${x - (x % 25) - SNAP_OFFSET}px`;
  else editor.style.left = `${x + (25 - (x % 25)) - SNAP_OFFSET}px`;

  if (y % 25 < 9) editor.style.top = `${y - (y % 25) - SNAP_OFFSET}px`;
  else editor.style.top = `${y + (25 - (y % 25)) - SNAP_OFFSET}px`;
};

function finish() {
  if (!current_editor) return;
  snap_editor(current_editor);
  current_editor.classList.remove("dragging");
  current_editor.dispatchEvent(new Event("moved"));
  current_editor = null;
  x_offset = null;
  y_offset = null;
  clicked = null;
}

let clicked;

function init(editor) {
  let rect = editor.getBoundingClientRect();
  editor.style.left = `${rect.left}px`;
  editor.style.top = `${rect.top}px`;
  editor.addEventListener("mousedown", (_) => {
    clicked = editor;
  });
  editor.addEventListener("mouseup", (_) => {
    clicked = null;
  });
  editor.addEventListener("mousemove", (e) => {
    if (current_editor) return;
    if (clicked !== editor) return;
    current_editor = editor;
    let x = left(current_editor);
    let y = top(current_editor);
    x_offset = e.clientX - x;
    y_offset = e.clientY - y;
    current_editor.classList.add("dragging");
    set_editor_position(x, y);
  });
}

window.addEventListener("mouseup", finish);
window.addEventListener("mousemove", (e) => {
  if (current_editor)
    set_editor_position(e.clientX - x_offset, e.clientY - y_offset);
});

window.addEventListener("blur", (_) => finish());
window.addEventListener("mouseleave", (_) => finish());
window.addEventListener("keydown", (_) => finish());

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
  });
});
