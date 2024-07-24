function handle_error({ detail: error }) {
  let keys = Object.keys(error);
  assert(keys.length === 1);
  let error_name = keys[0];
  let msg = error[error_name];
  this.setAttribute("data-error-type", error_name);
  this.setAttribute("data-error-msg", msg);
}

document.addEventListener("DOMContentLoaded", (_) => {
  let canvas = document.querySelector(".canvas");
  canvas.addEventListener("new-editor", ({ detail: { editor } }) => {
    editor.addEventListener("invoke-error", handle_error);
  });
});
