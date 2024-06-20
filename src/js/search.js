document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
  function clear() {
    input.classList.remove("active");
    input.innerText = "";
  }

  window.addEventListener("keydown", (e) => {
    if (e.key === "/") {
      input.classList.add("active");
      e.preventDefault();
      input.focus();
    } else if (e.key === "p" && e.metaKey) {
      e.preventDefault();
      if (input.classList.contains("active")) {
        clear();
      } else {
        input.classList.add("active");
        input.focus();
      }
    }
  });
  input.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();

      let html = await fetch(
        `http://localhost:8000/src/${input.textContent}/rule`,
      ).then((r) => r.text());

      let editor = document.createElement("div");
      editor.classList.add("--editor");
      editor.dataset.url = `http://localhost:8000/src/${input.textContent}`;
      editor.setAttribute("spellcheck", false);
      editor.innerHTML = html;

      let canvas = document.querySelector(".canvas");
      canvas.append(editor);
      canvas.dispatchEvent(new Event("new-editor"));
      clear();
    } else if (e.key === "Escape") {
      input.classList.remove("active");
    }
  });
});
