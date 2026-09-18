import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BluetoothXIcon` instead.
 *
 * @example
 * ```svelte
 * <BluetoothX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BluetoothX: Component<IconComponentProps, {}, "">;
type BluetoothX = ReturnType<typeof BluetoothX>;
export default BluetoothX;
