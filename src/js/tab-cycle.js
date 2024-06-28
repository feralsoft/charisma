window.addEventListener("keydown", (e) => {
  if (e.key === "Tab" && e.ctrlKey) {
    let current_editor = document
      .querySelector(".focused")
      .closest(".--editor");
    if (!current_editor) return;
    e.preventDefault();

    let tabbed_into_editor;
    if (e.shiftKey) {
      tabbed_into_editor = current_editor.previousElementSibling;
      if (!tabbed_into_editor?.classList?.contains("--editor"))
        tabbed_into_editor = document.querySelector(".--editor:last-child"); // last
    } else {
      tabbed_into_editor = current_editor.nextElementSibling;
      if (!tabbed_into_editor?.classList?.contains("--editor"))
        tabbed_into_editor = document.querySelector(".--editor:first-child"); // first
    }

    tabbed_into_editor.dispatchEvent(new Event("tab-into", { bubbles: true }));
  }
});
