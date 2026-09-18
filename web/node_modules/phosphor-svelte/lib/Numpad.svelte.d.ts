import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumpadIcon` instead.
 *
 * @example
 * ```svelte
 * <Numpad color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Numpad: Component<IconComponentProps, {}, "">;
type Numpad = ReturnType<typeof Numpad>;
export default Numpad;
