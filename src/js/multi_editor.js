document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    editor.addEventListener("reload", async (_) => {
      let new_rule = await fetch(`${editor.dataset.url}/rule`).then((r) =>
        r.text(),
      );
      editor.innerHTML = new_rule;
      editor.dispatchEvent(new Event("loaded"));
    });
  }
});
