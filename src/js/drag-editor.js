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
  drag_candidate = null;
  if (!state.editor) return;
  state.editor.style.position = "initial";
  state.editor.style.left = "initial";
  state.editor.style.top = "initial";
  state.editor.dispatchEvent(
    new CustomEvent("drag-finished", { detail: { position: state.position } }),
  );
  state = { editor: null, position: null, start_offset: null };
}

// mousedown is too eager, we have to ignore clicks
//
// the way to do this is by on first click, we track who is clicked.
// on mousemove, if the target is the same, it's a valid drag.
// if a "click" event happens between, stop dragging & revert.

let drag_candidate;

window.addEventListener("mousedown", (e) => {
  let editor = e.target.closest(".--editor");
  if (!editor) return;

  drag_candidate = editor;
});

window.addEventListener("mousemove", (e) => {
  if (drag_candidate && drag_candidate === e.target.closest(".--editor")) {
    // a drag is starting
    let editor = drag_candidate;
    drag_candidate = null;

    // unset the width
    editor.style.minWidth = "initial";
    editor.style.minHeight = "initial";

    // grab the position before we ungroup the editor
    let { top, left } = editor.getBoundingClientRect();

    let x_offset = px_var(document.body, "--x-offset"),
      y_offset = px_var(document.body, "--y-offset");

    // take it out of it's group & remove it if its empty
    let group = editor.closest(".--editor-group");
    let canvas = document.body.querySelector(".canvas");
    canvas.append(editor);
    if (!group?.querySelector(".--editor")) group.remove();

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
  } else if (state.editor) {
    // we are mid drag
    set_position(
      e.clientX - state.start_offset.x,
      e.clientY - state.start_offset.y,
    );
  }
});

window.addEventListener("mouseup", finish);
window.addEventListener("blur", finish);
