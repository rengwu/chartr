import { getContext, hasContext, setContext } from "svelte";

let contextKey = Symbol("phosphor-svelte");

export function setIconContext(value) {
  setContext(contextKey, value);
}

/**
 *
 * @returns {import("./shared").IconContextProps["values"]}
 */
export function getIconContext() {
  if (hasContext(contextKey)) {
    return getContext(contextKey);
  }

  return {};
}
