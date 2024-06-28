window.addEventListener("keydown", (e) => {
  if (e.key === "Tab" && e.ctrlKey) {
    let current_editor = document
      .querySelector(".focused")
      .closest(".--editor");
    if (!current_editor) return;
    e.preventDefault();
    let next_editor = current_editor.nextElementSibling;
    if (!next_editor?.classList?.contains("--editor"))
      next_editor = document.querySelector(".--editor"); // first

    next_editor.dispatchEvent(new Event("tab-into", { bubbles: true }));
  }
});
