export function once(elem, event_name, handler) {
  elem.addEventListener(event_name, function self(e) {
    handler(e);
    elem.removeEventListener(event_name, self);
  });
}
