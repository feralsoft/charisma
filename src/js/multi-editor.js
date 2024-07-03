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

function new_group({ x, y }) {
  let group = document.createElement("div");
  group.classList.add("--editor-group");
  group.style.setProperty("--x", x);
  group.style.setProperty("--y", y);
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
  let position = snap_position({
    x: width / 2,
    y: height / 2,
  });

  let group =
    find_map(document.elementsFromPoint(position.x, position.y), (elem) =>
      elem.closest(".--editor-group"),
    ) ?? new_group(position);

  group.append(editor);
}

function init(editor) {
  catch_links(editor);
  put_in_group(editor);
  editor.addEventListener("reload", (_) => reload(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
  });
});

window.assert = function (cond, msg) {
  if (!cond) {
    console.error(msg);
    debugger;
  }
};
