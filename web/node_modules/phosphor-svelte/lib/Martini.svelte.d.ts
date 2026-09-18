import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MartiniIcon` instead.
 *
 * @example
 * ```svelte
 * <Martini color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Martini: Component<IconComponentProps, {}, "">;
type Martini = ReturnType<typeof Martini>;
export default Martini;
