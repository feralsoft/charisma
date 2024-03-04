import compile from "@coil-lang/compiler/";

function coil() {
  return {
    name: "coil-lang",
    transform(src, id) {
      if (id.endsWith(".coil")) {
        return {
          code: compile(src, "@coil-lang/compiler/"),
          map: null,
        };
      }
    },
  };
}

export default coil;
