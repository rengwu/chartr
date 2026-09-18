import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UsbIcon` instead.
 *
 * @example
 * ```svelte
 * <Usb color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Usb: Component<IconComponentProps, {}, "">;
type Usb = ReturnType<typeof Usb>;
export default Usb;
