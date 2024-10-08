import { px_var } from "./helpers.js";

let state = { editor: null, position: null, start_offset: null };

function set_position(x, y) {
  state.position = { x, y };
  state.editor.style.left = `${x}px`;
  state.editor.style.top = `${y}px`;
}

function finish() {
  drag_candidate = null;
  if (!state.editor) return;
  state.editor.classList.remove("dragging");
  state.editor.style.position = "initial";
  state.editor.style.left = "initial";
  state.editor.style.top = "initial";
  state.editor.dispatchEvent(
    new CustomEvent("drag-finished", { detail: { position: state.position } }),
  );
  state = { editor: null, position: null, start_offset: null };
}

// mousedown is too eager, we have to ignore clicks so that links & buttons still work
//
// the way to do this is by on first click, we track who is clicked.
// on mousemove, if the target is the same, it's a valid drag.
// if a "click" event happens between, stop dragging & revert.

let drag_candidate;
let mousedown_position;

window.addEventListener("mousedown", (e) => {
  // shift is used for selecting text
  if (e.shiftKey) return;
  if (e.button !== 0) return;
  // we might be dragging a unit number up & down
  // like a px value.. fuck I hate the term "unit"
  if (document.body.classList.contains("dragging-unit")) return;

  let editor = e.target.closest(".--editor");
  if (!editor) return;

  mousedown_position = { x: e.clientX, y: e.clientY };

  drag_candidate = editor;
});

window.addEventListener("mousemove", (e) => {
  if (drag_candidate) {
    if (drag_candidate !== e.target.closest(".--editor")) return;
    let diff =
      Math.abs(e.clientX - mousedown_position.x) +
      Math.abs(e.clientY - mousedown_position.y);
    if (diff < 2) return;
    // a drag is starting
    let editor = drag_candidate;
    drag_candidate = null;

    editor.classList.add("dragging");
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

    // when we click on a rule that's been stretched by the group
    // eg.
    //
    // .test {} <- empty
    // .btn {
    //   font-size: 20px;
    // } <- this rule is wider then .test
    //
    // so when .test is in the same group as .btn, it'll appear wider
    // when it we take it out, we'll get the actual width
    //
    // if we click on a part of the rule that's outside of its natural size
    // and we drag, the rule won't be under the cursor
    //
    // so we can fix this by getting the diff between the cursor & the actual right of the rule
    // and correcting
    let { width: actual_width } = editor.getBoundingClientRect();
    let diff_right = e.clientX - (left + actual_width);

    editor.style.position = "absolute";

    state = {
      editor: editor,
      start_offset: {
        x: e.clientX - left + x_offset - Math.max(diff_right, 0),
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
