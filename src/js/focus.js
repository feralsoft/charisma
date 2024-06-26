const FOCUSABLE = [".--editor", '[data-kind="property"]'];

window.addEventListener("mousedown", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks
  for (let focused of document.querySelectorAll(".focused"))
    focused.classList.remove("focused");
  let elements = document.elementsFromPoint(e.clientX, e.clientY);

  for (let query of FOCUSABLE) {
    for (let elem of elements) {
      let result = elem.closest(query);
      if (!result) continue;
      result.classList.add("focused");
      break;
    }
  }
});
