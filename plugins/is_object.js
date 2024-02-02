export default function is_object(obj) {
  return Object.getPrototypeOf(obj) === Object.prototype;
}
