document.addEventListener("DOMContentLoaded", (_) => {
  let current_editor = null;
  let dx, dy;
  let set_position = (x, y) =>
    (current_editor.style = `position: absolute; top: ${y}; left: ${x}`);
  function finish() {
    if (!current_editor) return;
    let { left: x, top: y } = current_editor.getBoundingClientRect();
    current_editor.style.left = x - (x % 25) - 7;
    current_editor.style.top = y - (y % 25) - 7;
    current_editor.classList.remove("dragging");
    current_editor = null;
    dx = null;
    dy = null;
  }
  for (let editor of document.querySelectorAll(".--editor")) {
    editor.addEventListener("mousedown", (e) => {
      let rect = editor.getBoundingClientRect();
      dx = e.clientX - rect.left;
      dy = e.clientY - rect.top;
      current_editor = editor;
      current_editor.classList.add("dragging");
      set_position(rect.left, rect.top);
    });

    editor.addEventListener("mouseup", (_) => finish());
  }
  window.addEventListener("mousemove", (e) => {
    if (current_editor) set_position(e.clientX - dx, e.clientY - dy);
  });

  window.addEventListener("blur", (_) => finish());
  window.addEventListener("mouseleave", (_) => finish());
  window.addEventListener("keydown", (_) => finish());
});
