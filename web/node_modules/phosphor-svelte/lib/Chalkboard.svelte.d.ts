import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChalkboardIcon` instead.
 *
 * @example
 * ```svelte
 * <Chalkboard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Chalkboard: Component<IconComponentProps, {}, "">;
type Chalkboard = ReturnType<typeof Chalkboard>;
export default Chalkboard;
