import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StampIcon` instead.
 *
 * @example
 * ```svelte
 * <Stamp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stamp: Component<IconComponentProps, {}, "">;
type Stamp = ReturnType<typeof Stamp>;
export default Stamp;
