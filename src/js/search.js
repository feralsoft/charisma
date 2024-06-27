const { invoke } = window.__TAURI__.tauri;

window.add_editor = async function add_editor(selector) {
  let html = await invoke("render_rule", { selector });

  let editor = document.createElement("div");
  editor.classList.add("--editor");
  editor.dataset.selector = selector;
  editor.setAttribute("spellcheck", false);
  editor.innerHTML = html;

  let canvas = document.querySelector(".canvas");
  canvas.append(editor);
  document.querySelector(".--editor.focused")?.classList?.remove("focused");
  editor.classList.add("focused");
  canvas.dispatchEvent(new Event("new-editor"));
};

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
    if (e.key === "p" && e.metaKey) {
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

  options.addEventListener("mousedown", async (e) => {
    let selector = e.target.closest(".search-options > [data-kind]");
    if (!selector) return;
    await add_editor(selector.dataset.stringValue);
    clear();
  });

  input.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      await add_editor(options.firstElementChild.dataset.stringValue);
      clear();
    } else if (e.key === "Escape") {
      clear();
    } else {
      setTimeout(async () => {
        // setTimeout so that innerText gets populated
        if (input.innerText.trim() === "") {
          options.innerHTML = "";
        } else {
          let results = await invoke("search", { q: input.innerText });

          options.innerHTML = results.join("");
        }
      });
    }
  });
});
