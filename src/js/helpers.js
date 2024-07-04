export function px_var(elem, name, fallback = 0) {
  let value = elem.style.getPropertyValue(name);
  if (!value) return fallback;
  return Number(value.split("px")[0]);
}

export function find_map(iterable, fn) {
  for (let elem of iterable) {
    let result = fn(elem);
    if (result) return result;
  }
}
