let current_editor = null;
let dragging_board = false;
let board_old_x, board_old_y, start_x, start_y;
let dx, dy;
let set_position = (x, y) => {
  current_editor.style.left = x;
  current_editor.style.top = y;
};
function current_editor_x() {
  return Number(current_editor.style.left.split("px")[0]);
}

function current_editor_y() {
  return Number(current_editor.style.top.split("px")[0]);
}
function finish() {
  dragging_board = false;
  document.body.classList.remove("panning");
  if (!current_editor) return;
  let x = current_editor_x();
  let y = current_editor_y();
  current_editor.style.left = x - (x % 25) - 7;
  current_editor.style.top = y - (y % 25) - 7;
  current_editor.classList.remove("dragging");
  current_editor = null;
  dx = null;
  dy = null;
}
function init(editor) {
  let rect = editor.getBoundingClientRect();
  editor.style.left = rect.left;
  editor.style.top = rect.top;
  editor.addEventListener("mousedown", (e) => {
    if (
      !(
        e.target.dataset.kind === "rule" ||
        e.target.dataset.attr === "properties" ||
        e.target.dataset.attr === "inherited-properties"
      )
    )
      return;
    current_editor = editor;
    let x = current_editor_x();
    let y = current_editor_y();
    dx = e.clientX - x;
    dy = e.clientY - y;
    current_editor.classList.add("dragging");
    set_position(x, y);
  });
}

window.addEventListener("mousedown", (e) => {
  if (e.target.classList.contains("canvas")) {
    dragging_board = true;
    document.body.classList.add("panning");
    board_old_x = document.body.style.getPropertyValue("--dx");
    if (board_old_x) board_old_x = Number(board_old_x.split("px")[0]);
    else board_old_x = 0;
    board_old_y = document.body.style.getPropertyValue("--dy");
    if (board_old_y) board_old_y = Number(board_old_y.split("px")[0]);
    else board_old_y = 0;
    start_x = e.clientX;
    start_y = e.clientY;
  }
});
window.addEventListener("mouseup", finish);
window.addEventListener("mousemove", (e) => {
  if (dragging_board) {
    document.body.style.setProperty(
      "--dx",
      `${e.clientX - start_x + board_old_x}px`,
    );
    document.body.style.setProperty(
      "--dy",
      `${e.clientY - start_y + board_old_y}px`,
    );
  }
  if (current_editor) set_position(e.clientX - dx, e.clientY - dy);
});

window.addEventListener("blur", (_) => finish());
window.addEventListener("mouseleave", (_) => finish());
window.addEventListener("keydown", (_) => finish());

document.addEventListener("DOMContentLoaded", (_) => {
  document.body.classList.add("draggable-editor-prototype");
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
