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

function x_offset() {
  return Number(
    document.body.style.getPropertyValue("--x-offset").split("px")[0],
  );
}

function y_offset() {
  return Number(
    document.body.style.getPropertyValue("--y-offset").split("px")[0],
  );
}

function new_group(editor) {
  let group = document.createElement("div");
  group.classList.add("--editor-group");

  let { width, height } = document.body.getBoundingClientRect();

  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  let pos = snap_position({
    x: width / 2 - editor_width / 3,
    y: height / 2 - editor_height / 3,
  });

  group.style.setProperty("--x", pos.x);
  group.style.setProperty("--y", pos.y);
  document.querySelector(".canvas").append(group);
  return group;
}

window.find_map = function (iterable, fn) {
  for (let elem of iterable) {
    let result = fn(elem);
    if (result) return result;
  }
};

function put_in_group(editor) {
  let { width, height } = document.body.getBoundingClientRect();

  let group =
    find_map(document.elementsFromPoint(width / 2, height / 2), (elem) =>
      elem.closest(".--editor-group"),
    ) ?? new_group(editor);

  group.append(editor);
}

const SNAP_OFFSET = 4;

function snap_size(editor) {
  let { width, height } = editor.getBoundingClientRect();
  width = width + (25 - (width % 25)) - SNAP_OFFSET;
  height = height + (25 - (height % 25)) - SNAP_OFFSET;

  editor.style.width = `${width}px`;
  editor.style.height = `${height}px`;
}

function snap_position({ x, y }) {
  if (x % 25 < 9) x = x - (x % 25) - SNAP_OFFSET;
  else x = x + (25 - (x % 25)) - SNAP_OFFSET;

  if (y % 25 < 9) y = y - (y % 25) - SNAP_OFFSET;
  else y = y + (25 - (y % 25)) - SNAP_OFFSET;

  return { x, y };
}

function snap_and_group(editor) {
  put_in_group(editor);
  snap_size(editor);
}

function init(editor) {
  catch_links(editor);
  snap_and_group(editor);
  editor.addEventListener("reload", (_) => reload(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("move-to-new-group", (_) => snap_and_group(editor));
  });
});

window.assert = function (cond, msg) {
  if (!cond) {
    console.error(msg);
    debugger;
  }
};
