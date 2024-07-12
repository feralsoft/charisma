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

export function* zip(a, b) {
  let iter_a = a[Symbol.iterator]();
  let iter_b = b[Symbol.iterator]();
  let iter_a_result, iter_b_result;
  do {
    iter_a_result = iter_a.next();
    iter_b_result = iter_b.next();
    yield [iter_a_result.value, iter_b_result.value];
  } while (!iter_a_result.done && !iter_b_result.done);
}
