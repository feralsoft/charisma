function catch_links(editor) {
  for (let a of editor.querySelectorAll("a")) {
    // a.addEventListener("click", (e) => );
    a.addEventListener("click", async (e) => {
      e.preventDefault();
      let url = new URL(a.href);
      url.pathname += "/rule";
      let new_rule = await fetch(url).then((r) => r.text());
      editor.innerHTML = new_rule;
      editor.dataset.url = a.href;
      editor.dispatchEvent(new Event("loaded"));
    });
  }
}
function init(editor) {
  catch_links(editor);
  editor.addEventListener("reload", async (_) => {
    let new_rule = await fetch(url_for(editor, "/rule")).then((r) => r.text());
    editor.innerHTML = new_rule;
    editor.dataset.url = url_for(editor);
    catch_links(editor);
    editor.dispatchEvent(new Event("loaded"));
  });
}
document.addEventListener("DOMContentLoaded", (_) => {
  for (let editor of document.querySelectorAll(".--editor")) {
    init(editor);
  }
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", (_) => {
    let editor = document.querySelector(".--editor:last-child");
    init(editor);
  });
});

window.url_for = function (editor, sub_path = "", search = "") {
  let url = new URL(
    decodeURIComponent(editor.dataset.url).replaceAll("+", " "),
  );
  url.pathname += sub_path;
  url.search = search;
  return url;
};
