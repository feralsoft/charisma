document.addEventListener("DOMContentLoaded", (_) => {
  let input = document.querySelector(".search");
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      location.pathname = `src/${input.textContent}`;
    }
  });
});
