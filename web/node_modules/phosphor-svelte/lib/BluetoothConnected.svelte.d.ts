import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BluetoothConnectedIcon` instead.
 *
 * @example
 * ```svelte
 * <BluetoothConnected color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BluetoothConnected: Component<IconComponentProps, {}, "">;
type BluetoothConnected = ReturnType<typeof BluetoothConnected>;
export default BluetoothConnected;
