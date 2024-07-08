import { find, num_var } from "./helpers.js";

const { invoke } = window.__TAURI__.tauri;

let reload_lock = new Map();

async function reload(editor) {
  if (reload_lock.get(editor)) return;
  reload_lock.set(editor, true);
  for (let editor_ of document.querySelectorAll(".--editor")) {
    if (editor_ === editor) continue;
    editor_.dispatchEvent(
      new CustomEvent("reload", { detail: { src: "reload-siblings" } }),
    );
  }

  let new_rule = await invoke("render_rule", {
    selector: editor.dataset.selector,
  });

  editor.innerHTML = new_rule;
  catch_links(editor);
  editor.dispatchEvent(new Event("loaded"));
  reload_lock.delete(editor);
}

function catch_links(editor) {
  for (let a of editor.querySelectorAll("a"))
    a.addEventListener("click", (e) => {
      e.preventDefault();
      add_editor(a.getAttribute("href"));
    });
}

function new_group(position) {
  let group = document.createElement("div");
  group.classList.add("--editor-group");

  // - 25 since the group has a padding of 25px
  let pos = snap_position({ x: position.x - 25, y: position.y - 25 });

  group.style.setProperty("--x", pos.x);
  group.style.setProperty("--y", pos.y);
  document.querySelector(".canvas").append(group);
  return group;
}

function is_overlapping(group, editor, editor_position) {
  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();
  let editor_top = editor_position.y;
  let editor_left = editor_position.x;
  let editor_right = editor_left + editor_width;
  let editor_bottom = editor_top + editor_height;

  let { width: group_width, height: group_height } =
    group.getBoundingClientRect();
  let group_left = num_var(group, "--x");
  let group_top = num_var(group, "--y");
  let group_bottom = group_top + group_height;
  let group_right = group_left + group_width;

  return (
    group_bottom > editor_top &&
    editor_bottom > group_top &&
    editor_left < group_right &&
    editor_right > group_left
  );
}

function put_in_group(editor, position) {
  // position is top-left
  //
  // and we search for --editor-group with that position
  // but this fails easily in this case
  //
  // |----------------|
  // | dropped-editor |
  // |  |----------------|
  // |  | old  editor    |
  // |  |----------------|
  // |----------------|
  //
  // ^ here I dropped an editor over-top of an existing editor `old editor`
  // but since top left isn't within old editor, it won't snap..
  //
  // intersection observer requires you to register all the elements to watch afaik
  // this seems hard to maintain, so for now we will just loop over all elements & check overlap

  let group =
    find(document.querySelectorAll(".--editor-group"), (group) =>
      is_overlapping(group, editor, position),
    ) ?? new_group(position);

  group.append(editor);
}

const SNAP_OFFSET = 4;

function snap_size(editor) {
  editor.style.minWidth = "initial";
  editor.style.minHeight = "initial";
  let { width, height } = editor.getBoundingClientRect();
  width = width + (25 - (width % 25)) - SNAP_OFFSET;
  height = height + (25 - (height % 25)) - SNAP_OFFSET;

  editor.style.minWidth = `${width}px`;
  editor.style.minHeight = `${height}px`;
}

export function snap_position({ x, y }) {
  if (x % 25 < 9) x = x - (x % 25) - SNAP_OFFSET;
  else x = x + (25 - (x % 25)) - SNAP_OFFSET;

  if (y % 25 < 9) y = y - (y % 25) - SNAP_OFFSET;
  else y = y + (25 - (y % 25)) - SNAP_OFFSET;

  return { x, y };
}

function snap_and_group(editor, position) {
  put_in_group(editor, position);
  snap_size(editor);
}

function init(editor) {
  catch_links(editor);

  let { width: body_width, height: body_height } =
    document.body.getBoundingClientRect();

  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  snap_and_group(editor, {
    x: body_width / 2 - editor_width / 3,
    y: body_height / 2 - editor_height / 3,
  });
  editor.addEventListener("reload", (_) => reload(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("drag-finished", ({ detail: { position } }) =>
      snap_and_group(editor, position),
    );
    editor.addEventListener("blur", (_) => snap_size(editor));
    editor.addEventListener("loaded", (_) => snap_size(editor));
    editor.addEventListener("focus", (_) => snap_size(editor));
    editor.addEventListener("click", (_) => snap_size(editor));
  });
});

window.assert = function (cond, msg) {
  if (!cond) {
    console.error(msg);
    debugger;
  }
};
