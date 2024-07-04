function close(editor) {
  let close_btn = document.createElement("button");
  close_btn.classList.add("close");
  close_btn.addEventListener("mousedown", (_) => {
    let group = editor.closest(".--editor-group");
    editor.remove();
    if (!group.querySelector(".--editor")) group.remove();
  });
  return close_btn;
}

function minimize(editor) {
  let minimize_btn = document.createElement("button");
  minimize_btn.classList.add("minimize");
  minimize_btn.addEventListener("mousedown", (_) => {
    minimize_btn.dataset.selector = editor.querySelector(
      '[data-attr="selector"] > [data-kind]',
    ).dataset.stringValue;
    minimize_btn.classList.toggle("active");
  });
  return minimize_btn;
}

function info() {
  let info_btn = document.createElement("button");
  info_btn.classList.add("info");
  info_btn.addEventListener("mousedown", (_) => {
    info_btn.classList.toggle("active");
  });
  return info_btn;
}

function menu(editor) {
  let menu_elem = document.createElement("div");
  menu_elem.classList.add("menu");

  menu_elem.append(close(editor));
  menu_elem.append(minimize(editor));
  menu_elem.append(info());
  return menu_elem;
}

function init(editor) {
  editor.prepend(menu(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  });
});
