import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GraphIcon` instead.
 *
 * @example
 * ```svelte
 * <Graph color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Graph: Component<IconComponentProps, {}, "">;
type Graph = ReturnType<typeof Graph>;
export default Graph;
