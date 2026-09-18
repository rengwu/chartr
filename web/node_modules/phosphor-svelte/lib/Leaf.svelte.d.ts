import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LeafIcon` instead.
 *
 * @example
 * ```svelte
 * <Leaf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Leaf: Component<IconComponentProps, {}, "">;
type Leaf = ReturnType<typeof Leaf>;
export default Leaf;
