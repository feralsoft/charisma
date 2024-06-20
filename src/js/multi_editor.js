let reloading = new Map();

async function reload(editor, base_url) {
  // if (reloading.get(editor)) return;
  // reloading.set(editor, true);
  // for (let editor_ of document.querySelectorAll(".--editor"))
  //   editor_.dispatchEvent(new Event("reload"));
  let url = new URL(base_url);
  url.pathname += "/rule";
  let new_rule = await fetch(url).then((r) => r.text());
  editor.innerHTML = new_rule;
  editor.dataset.url = url;
  catch_links(editor);
  editor.dispatchEvent(new Event("loaded"));
  // reloading.set(editor, false);
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

function init(editor) {
  catch_links(editor);
  let { width, height } = document.body.getBoundingClientRect();
  let { width: editor_width, height: editor_height } =
    editor.getBoundingClientRect();
  editor.style.left = `${width / 2 - editor_width / 2 - x_offset()}px`;
  editor.style.top = `${height / 2 - editor_height / 2 - y_offset()}px`;
  snap_editor(editor);
  editor.addEventListener("reload", (_) => reload(editor, url_for(editor)));
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) init(editor);
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
