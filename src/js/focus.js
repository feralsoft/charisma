window.addEventListener("click", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks
  document.querySelector(".--editor.focused")?.classList?.remove("focused");
  let elem = document.elementFromPoint(e.clientX, e.clientY);
  let editor = elem.closest(".--editor");
  if (!editor) return;
  editor.classList.add("focused");
});
