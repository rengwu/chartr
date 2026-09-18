import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TreeIcon` instead.
 *
 * @example
 * ```svelte
 * <Tree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tree: Component<IconComponentProps, {}, "">;
type Tree = ReturnType<typeof Tree>;
export default Tree;
