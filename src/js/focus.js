window.addEventListener("mousedown", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks
  for (let focused of document.querySelectorAll(".focused"))
    focused.classList.remove("focused");
  let elements = document.elementsFromPoint(e.clientX, e.clientY);

  for (let elem of elements) {
    let result = elem.closest(".--editor");
    if (!result) continue;
    result.classList.add("focused");
    break;
  }
});

window.addEventListener("dblclick", (e) => {
  if (!e.isTrusted) return; // don't trust simulated mouse clicks
  for (let focused of document.querySelectorAll(".focused"))
    focused.classList.remove("focused");
  let elements = document.elementsFromPoint(e.clientX, e.clientY);

  for (let elem of elements) {
    let result = elem.closest('[data-kind="property"]');
    if (!result) continue;
    result.classList.add("focused");
    break;
  }
});
