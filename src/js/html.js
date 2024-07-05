export function h(kind, attrs, ...children) {
  let elem = document.createElement(kind);
  for (let [key, value] of Object.entries(attrs)) {
    if (key.startsWith("@")) {
      let [_, event_name] = key.split("@");
      assert(typeof value === "function");
      elem.addEventListener(event_name, value);
    } else {
      elem.setAttribute(key, value);
    }
  }
  elem.append(...children);
  return elem;
}
