export let h = new Proxy(
  {},
  {
    get(_, kind) {
      return (attrs, ...children) => {
        let elem = document.createElement(kind);
        for (let [key, value] of Object.entries(attrs)) {
          if (typeof key === "symbol") continue;
          if (key.startsWith("@")) {
            let [_, event_name] = key.split("@");
            assert(typeof value === "function");
            elem.addEventListener(event_name, value);
          } else {
            elem.setAttribute(key, value);
          }
        }
        elem.append(...children);
        if (attrs[modifiers.on_mount]) {
          setTimeout(() => {
            attrs[modifiers.on_mount].call(elem);
          });
        }
        return elem;
      };
    },
  },
);

export let modifiers = {
  on_mount: Symbol("on_mount"),
};
