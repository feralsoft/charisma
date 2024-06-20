document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
  let container = input.closest(".search-box");
  let options = document.createElement("div");
  options.classList.add("search-options");
  container.append(options);
  function clear() {
    input.classList.remove("active");
    input.innerText = "";
    options.innerHTML = "";
  }

  window.addEventListener("keydown", async (e) => {
    if (e.key === "/" || (e.key === "p" && e.metaKey)) {
      e.preventDefault();
      if (input.classList.contains("active")) {
        clear();
      } else {
        input.classList.add("active");
        input.focus();
      }
    }
  });

  window.addEventListener("mousedown", (e) => {
    if (!e.target.closest(".search-box")) clear();
  });

  options.addEventListener("mousedown", (e) => {
    let selector = e.target.closest(".search-options > [data-kind]");
    if (!selector) return;
    add_editor(selector.dataset.stringValue);
  });

  async function add_editor(selector) {
    let html = await fetch(`http://localhost:8000/src/${selector}/rule`).then(
      (r) => r.text(),
    );

    let editor = document.createElement("div");
    editor.classList.add("--editor");
    editor.dataset.url = `http://localhost:8000/src/${selector}`;
    editor.setAttribute("spellcheck", false);
    editor.innerHTML = html;

    let canvas = document.querySelector(".canvas");
    canvas.append(editor);
    canvas.dispatchEvent(new Event("new-editor"));
    clear();
  }

  input.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      add_editor(options.firstElementChild.dataset.stringValue);
    } else if (e.key === "Escape") {
      clear();
    } else {
      setTimeout(async () => {
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
        } else {
          let results = await fetch(
            `http://localhost:8000/search/${input.innerText}`,
          ).then((r) => r.json());
          options.innerHTML = results.join("");
        }
      });
    }
  });
});
