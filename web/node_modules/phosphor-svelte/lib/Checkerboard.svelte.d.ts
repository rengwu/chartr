import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckerboardIcon` instead.
 *
 * @example
 * ```svelte
 * <Checkerboard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Checkerboard: Component<IconComponentProps, {}, "">;
type Checkerboard = ReturnType<typeof Checkerboard>;
export default Checkerboard;
