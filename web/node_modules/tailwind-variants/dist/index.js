import { getTailwindVariants, state } from './chunk-RZF76H2U.js';
export { defaultConfig } from './chunk-RZF76H2U.js';
import { cx, isEmptyObject } from './chunk-LQJYWU4O.js';
export { cx } from './chunk-LQJYWU4O.js';
import { twMerge, extendTailwindMerge } from 'tailwind-merge';

var createTwMerge = (cachedTwMergeConfig) => {
  return isEmptyObject(cachedTwMergeConfig) ? twMerge : extendTailwindMerge({
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
  const base = cx(classnames);
  if (!base || !(config?.twMerge ?? true)) return base;
  if (!state.cachedTwMerge || state.didTwMergeConfigChange) {
    state.didTwMergeConfigChange = false;
    state.cachedTwMerge = createTwMerge(state.cachedTwMergeConfig);
  }
  return state.cachedTwMerge(base) || void 0;
};
var cn = (...classnames) => {
  return executeMerge(classnames, {});
};
var cnMerge = (...classnames) => {
  return (config) => executeMerge(classnames, config);
};

// src/index.js
var { createTV, tv } = getTailwindVariants(cnMerge);

export { cn, cnMerge, createTV, tv };
