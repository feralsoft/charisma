import { px_var } from "./helpers.js";

let state = {
  editor: null,
  position: null,
  start_offset: null,
};

function set_position(x, y) {
  state.position = { x, y };
  state.editor.style.left = `${x}px`;
  state.editor.style.top = `${y}px`;
}

function finish() {
  if (!state.editor) return;
  state.editor.style.position = "initial";
  state.editor.style.left = "initial";
  state.editor.style.top = "initial";
  state.editor.dispatchEvent(
    new CustomEvent("drag-finished", { detail: { position: state.position } }),
  );
  state = { editor: null, position: null, start_offset: null };
}

window.addEventListener("mousedown", (e) => {
  let editor = e.target.closest(".--editor");
  if (!editor) return;

  // grab the position before we ungroup the editor
  let { top, left } = editor.getBoundingClientRect();

  let x_offset = px_var(document.body, "--x-offset"),
    y_offset = px_var(document.body, "--y-offset");

  // take it out of it's group & remove it if its empty
  let group = editor.closest(".--editor-group");
  let canvas = document.body.querySelector(".canvas");
  canvas.append(editor);
  if (!group.querySelector(".--editor")) group.remove();

  editor.style.position = "absolute";

  state = {
    editor: editor,
    start_offset: {
      x: e.clientX - left + x_offset,
      y: e.clientY - top + y_offset,
    },
  };

  set_position(
    e.clientX - state.start_offset.x,
    e.clientY - state.start_offset.y,
  );
});

window.addEventListener("mousemove", (e) => {
  if (!state.editor) return;
  set_position(
    e.clientX - state.start_offset.x,
    e.clientY - state.start_offset.y,
  );
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);

// document.addEventListener("DOMContentLoaded", (_) => {
//   let canvas = document.querySelector(".canvas");
//   canvas.addEventListener("new-editor", ({ detail: editor }) => {
//     init(editor);
//     editor.addEventListener("loaded", (_) => init(editor));
//   });
// });
