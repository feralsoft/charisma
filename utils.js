window.primitive_type = Symbol("primitive_type");

Object.prototype[primitive_type] = function () {
  return typeof this;
};
