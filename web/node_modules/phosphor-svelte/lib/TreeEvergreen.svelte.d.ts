import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TreeEvergreenIcon` instead.
 *
 * @example
 * ```svelte
 * <TreeEvergreen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TreeEvergreen: Component<IconComponentProps, {}, "">;
type TreeEvergreen = ReturnType<typeof TreeEvergreen>;
export default TreeEvergreen;
