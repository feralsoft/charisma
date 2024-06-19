function close(editor) {
  let close_btn = document.createElement("button");
  close_btn.classList.add("close");
  close_btn.addEventListener("mousedown", (_) => editor.remove());
  return close_btn;
}

function minimize() {
  let close_btn = document.createElement("button");
  close_btn.classList.add("minimize");
  close_btn.addEventListener("mousedown", (_) =>
    close_btn.classList.toggle("active"),
  );
  return close_btn;
}

function menu(editor) {
  let menu_elem = document.createElement("div");
  menu_elem.classList.add("menu");

  menu_elem.append(close(editor));
  menu_elem.append(minimize());
  return menu_elem;
}

function init(editor) {
  editor.prepend(menu(editor));
}

document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
    editor.addEventListener("loaded", (_) => init(editor));
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});
