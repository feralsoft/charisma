export function px_var(elem, name, fallback = 0) {
  let value = elem.style.getPropertyValue(name);
  if (!value) return fallback;
  return Number(value.split("px")[0]);
}
