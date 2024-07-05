function close(group) {
  let close_btn = document.createElement("button");
  close_btn.classList.add("close");
  close_btn.addEventListener("click", (_) => group.remove());
  return close_btn;
}

function minimize() {
  let minimize_btn = document.createElement("button");
  minimize_btn.classList.add("minimize");
  minimize_btn.addEventListener("click", (_) => {
    minimize_btn.classList.toggle("active");
  });
  return minimize_btn;
}
function menu(group) {
  let menu_elem = document.createElement("div");
  menu_elem.classList.add("menu");

  menu_elem.append(close(group));
  menu_elem.append(minimize(group));
  return menu_elem;
}

function init(group) {
  if (group.querySelector(":scope > .menu")) return;
  group.prepend(menu(group));
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: editor }) => {
    init(editor.closest(".--editor-group"));
    editor.addEventListener("drag-finished", () =>
      init(editor.closest(".--editor-group")),
    );
  });
});
