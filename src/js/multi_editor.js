let reload_lock = new Map();

async function reload(editor, base_url) {
  if (reload_lock.get(editor)) return;
  reload_lock.set(editor, true);
  for (let editor_ of document.querySelectorAll(".--editor"))
    editor_.dispatchEvent(new Event("reload"));
  let url = new URL(base_url);
  url.pathname += "/rule";
  let new_rule = await fetch(url).then((r) => r.text());
  editor.innerHTML = new_rule;
  editor.dataset.url = base_url;
  catch_links(editor);
  editor.dispatchEvent(new Event("loaded"));
  reload_lock.delete(editor);
}

function catch_links(editor) {
  for (let a of editor.querySelectorAll("a")) {
    a.addEventListener("click", async (e) => {
      e.preventDefault();
      reload(editor, a.href);
    });
  }
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

function editor_at({ x, y }) {
  for (let elem of document.elementsFromPoint(x, y)) {
    let editor = elem.closest(".--editor");
    if (editor) return editor;
  }
}

function find_position_for(editor) {
  let { width, height } = document.body.getBoundingClientRect();
  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();

  let position = {
    x: width / 2 - editor_width / 2,
    y: height / 2 - editor_height / 2,
  };

  // ugghh .. this is broken & ugly. :)
  let elem;
  if ((elem = editor_at(position))) {
    let { bottom, right, left, top } = elem.getBoundingClientRect();
    // go right
    position.x = right + 50;
    position.y = top;
    if (editor_at(position)) {
      // try moving down
      position.y = bottom + 50;
      position.x = left;
      // move bottom right
      if (editor_at(position)) {
        position.x = right + 50;
        if (editor_at(position)) {
          // go left
          position.x = left - editor_width;
          position.y = top;
          if (editor_at(position)) {
            // go up
            position.y = top - editor_height - 50;
            position.x = left;
            if (editor_at(position)) {
              // up right
              position.x = right + 50;
              if (editor_at(position)) {
                // up left
                position.x = left - editor_width;
                if (editor_at(position)) {
                  position.y = bottom + 50;
                }
              }
            }
          }
        }
      }
    }
  }
  return position;
}

function init(editor) {
  catch_links(editor);
  let position = find_position_for(editor);
  editor.style.left = `${position.x - x_offset()}px`;
  editor.style.top = `${position.y - y_offset()}px`;
  snap_editor(editor);
  editor.addEventListener("reload", (_) => reload(editor, url_for(editor)));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});

window.url_for = function (editor, sub_path = "", search = "") {
  let url = new URL(
    decodeURIComponent(editor.dataset.url).replaceAll("+", " "),
  );
  url.pathname += sub_path;
  url.search = search;
  return url;
};

window.assert = function (cond, msg) {
  if (!cond) {
    debugger;
  }
};
