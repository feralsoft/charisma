function input() {
  let elem = document.createElement("input");
  elem.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      let [name, value] = e.target.value.split(":");
      value = value.replaceAll(";", "");
      fetch(`${location.pathname}/insert`, {
        method: "POST",
        body: JSON.stringify({ name, value }),
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
