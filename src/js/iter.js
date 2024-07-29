export function find_map(iterable, fn) {
  for (let elem of iterable) {
    let result = fn(elem);
    if (result) return result;
  }
}

export function find(iterable, fn) {
  for (let elem of iterable) {
    if (fn(elem)) return elem;
  }
}
