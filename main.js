window.try__b = function (lambda) {
  try {
    return lambda();
  } catch (e) {
    if (!(e instanceof Error)) e = new Error(e);
    return e;
  }
};

import "./index.css";
import "./a.css";
import "./b.css";
import "./main.coil";
