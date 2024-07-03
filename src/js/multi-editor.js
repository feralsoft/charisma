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

function editor_at({ x, y }, self) {
  for (let elem of document.elementsFromPoint(x, y)) {
    let editor = elem.closest(".--editor");
    if (editor && editor !== self) return editor;
  }
}

const POSITIONS = [
  [1, 0],
  [1, 1],
  [0, 1],
  [-1, 1],
  [-1, 0],
  [-1, -1],
  [0, -1],
  [1, -1],
];

function find_position_for(editor) {
  let { width, height } = document.body.getBoundingClientRect();
  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  let position = {
    x: width / 2 - editor_width / 2,
    y: height / 2 - editor_height / 2,
  };

  if (!editor_at(position, editor)) return position;

  for (let [dir_x, dir_y] of POSITIONS) {
    let new_position = {
      x: position.x + dir_x * editor_width,
      y: position.y + dir_y * editor_height,
    };

    if (!editor_at(new_position, editor)) return new_position;
  }
  return position;
}

function init(editor) {
  catch_links(editor);
  let position = find_position_for(editor);
  editor.style.left = `${position.x - x_offset()}px`;
  editor.style.top = `${position.y - y_offset()}px`;
  snap_editor(editor);
  editor.addEventListener("reload", (_) => reload(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});

window.assert = function (cond, msg) {
  if (!cond) {
    debugger;
  }
};
