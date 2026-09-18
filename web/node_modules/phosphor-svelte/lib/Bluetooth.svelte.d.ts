import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BluetoothIcon` instead.
 *
 * @example
 * ```svelte
 * <Bluetooth color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bluetooth: Component<IconComponentProps, {}, "">;
type Bluetooth = ReturnType<typeof Bluetooth>;
export default Bluetooth;
