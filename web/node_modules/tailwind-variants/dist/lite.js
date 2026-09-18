import { getTailwindVariants } from './chunk-RZF76H2U.js';
export { defaultConfig } from './chunk-RZF76H2U.js';
import { cx } from './chunk-LQJYWU4O.js';
export { cx } from './chunk-LQJYWU4O.js';

// src/lite.js
var cnAdapter = (...classnames) => {
  return (_config) => {
    const base = cx(classnames);
    return base || void 0;
  };
};
var { createTV, tv } = getTailwindVariants(cnAdapter);

export { cnAdapter as cn, cnAdapter, createTV, tv };
