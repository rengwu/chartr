import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KeyboardIcon` instead.
 *
 * @example
 * ```svelte
 * <Keyboard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Keyboard: Component<IconComponentProps, {}, "">;
type Keyboard = ReturnType<typeof Keyboard>;
export default Keyboard;
