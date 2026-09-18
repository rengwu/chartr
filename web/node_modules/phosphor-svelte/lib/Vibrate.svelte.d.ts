import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VibrateIcon` instead.
 *
 * @example
 * ```svelte
 * <Vibrate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Vibrate: Component<IconComponentProps, {}, "">;
type Vibrate = ReturnType<typeof Vibrate>;
export default Vibrate;
