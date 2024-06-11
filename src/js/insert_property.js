function input() {
  let elem = document.createElement("input");
  elem.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      fetch(`${location.pathname}`, {
        method: "POST",
        body: e.target.value,
      }).finally((_) => location.reload());
    }
  });
  return elem;
}

document.addEventListener("DOMContentLoaded", () => {
  let properties = document.querySelector(
    "[data-kind=rule] > [data-attr=properties]",
  );

  properties.append(input());
});
