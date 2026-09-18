'use strict';

var chunk52Z2HSI4_cjs = require('./chunk-52Z2HSI4.cjs');
var chunk2JY7EID6_cjs = require('./chunk-2JY7EID6.cjs');

// src/lite.js
var cnAdapter = (...classnames) => {
  return (_config) => {
    const base = chunk2JY7EID6_cjs.cx(classnames);
    return base || void 0;
  };
};
var { createTV, tv } = chunk52Z2HSI4_cjs.getTailwindVariants(cnAdapter);

Object.defineProperty(exports, "defaultConfig", {
  enumerable: true,
  get: function () { return chunk52Z2HSI4_cjs.defaultConfig; }
});
Object.defineProperty(exports, "cx", {
  enumerable: true,
  get: function () { return chunk2JY7EID6_cjs.cx; }
});
exports.cn = cnAdapter;
exports.cnAdapter = cnAdapter;
exports.createTV = createTV;
exports.tv = tv;
