export const is_match = Symbol("is_match");

String.prototype[is_match] = function (str) {
  if (str.startsWith(this)) {
    return { result: this, skip: this.length };
  } else {
    return false;
  }
};

RegExp.prototype[is_match] = function (str, group = 0) {
  let result = str.match(this);
  if (result === null || result.index !== 0) {
    return false;
  } else {
    return { result: result[group], skip: result[0].length };
  }
};

let GeneratorFunction = function* () {}.constructor;

export const run = Symbol("run");

GeneratorFunction.prototype[run] = function (string, ...args) {
  let gen = this(...args);
  let result = null;
  while (true) {
    let { value, done } = gen.next(result);
    if (done) {
      return [value, string];
    } else {
      let r = value[run](string, result);
      if (typeof r !== "undefined") {
        result = r[0];
        string = r[1];
      }
    }
  }
};
let DEBUG_PARSER = true;

export class Chomp {
  constructor(pattern) {
    this.pattern = pattern;
  }
  [run](string) {
    let result = this.pattern[is_match](string);
    if (result) {
      return [true, string.slice(result.skip)];
    }
    if (DEBUG_PARSER) {
      console.log(this, { pattern, string });
      throw "parse error";
    }
  }
}
export class Capture {
  constructor(pattern, group = 0) {
    this.pattern = pattern;
    this.group = group;
  }
  [run](string) {
    let r = this.pattern[is_match](string);
    if (r) {
      let { result, skip } = r;
      return [result, string.slice(skip)];
    }
    if (DEBUG_PARSER) {
      console.log(this, { pattern, group, string });
      throw "parse error";
    }
  }
}
export class TrimmedSafe {
  constructor(parser) {
    this.parser = parser;
  }
  [run](string) {
    let [result, _] = this.parser[run](string.trimLeft());
    return [result, string];
  }
}
export class Trimmed {
  constructor(parser) {
    this.parser = parser;
  }
  [run](string) {
    let [result, s] = this.parser[run](string.trimLeft());
    return [result, s.trimLeft()];
  }
}
export class TrimBefore {
  constructor(parser) {
    this.parser = parser;
  }
  [run](string) {
    let [result, s] = this.parser[run](string.trimLeft());
    return [result, s];
  }
}
export class Test {
  constructor(pattern) {
    this.pattern = pattern;
  }
  [run](string) {
    return [this.pattern[is_match](string), string];
  }
}
export class Trim {
  [run](string) {
    return [null, string.trimLeft()];
  }
}
export class Debug {
  [run](string, state) {
    return [[state, string], string];
  }
}
