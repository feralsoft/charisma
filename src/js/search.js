document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
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
      input.innerText = "";
    }
  });
});
