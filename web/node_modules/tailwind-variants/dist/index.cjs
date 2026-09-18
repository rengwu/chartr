'use strict';

var chunk52Z2HSI4_cjs = require('./chunk-52Z2HSI4.cjs');
var chunk2JY7EID6_cjs = require('./chunk-2JY7EID6.cjs');
var tailwindMerge = require('tailwind-merge');

var createTwMerge = (cachedTwMergeConfig) => {
  return chunk2JY7EID6_cjs.isEmptyObject(cachedTwMergeConfig) ? tailwindMerge.twMerge : tailwindMerge.extendTailwindMerge({
    ...cachedTwMergeConfig,
    extend: {
      theme: cachedTwMergeConfig.theme,
      classGroups: cachedTwMergeConfig.classGroups,
      conflictingClassGroupModifiers: cachedTwMergeConfig.conflictingClassGroupModifiers,
      conflictingClassGroups: cachedTwMergeConfig.conflictingClassGroups,
      ...cachedTwMergeConfig.extend
    }
  });
};
var executeMerge = (classnames, config) => {
  const base = chunk2JY7EID6_cjs.cx(classnames);
  if (!base || !(config?.twMerge ?? true)) return base;
  if (!chunk52Z2HSI4_cjs.state.cachedTwMerge || chunk52Z2HSI4_cjs.state.didTwMergeConfigChange) {
    chunk52Z2HSI4_cjs.state.didTwMergeConfigChange = false;
    chunk52Z2HSI4_cjs.state.cachedTwMerge = createTwMerge(chunk52Z2HSI4_cjs.state.cachedTwMergeConfig);
  }
  return chunk52Z2HSI4_cjs.state.cachedTwMerge(base) || void 0;
};
var cn = (...classnames) => {
  return executeMerge(classnames, {});
};
var cnMerge = (...classnames) => {
  return (config) => executeMerge(classnames, config);
};

// src/index.js
var { createTV, tv } = chunk52Z2HSI4_cjs.getTailwindVariants(cnMerge);

Object.defineProperty(exports, "defaultConfig", {
  enumerable: true,
  get: function () { return chunk52Z2HSI4_cjs.defaultConfig; }
});
Object.defineProperty(exports, "cx", {
  enumerable: true,
  get: function () { return chunk2JY7EID6_cjs.cx; }
});
exports.cn = cn;
exports.cnMerge = cnMerge;
exports.createTV = createTV;
exports.tv = tv;
