import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BluetoothSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <BluetoothSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BluetoothSlash: Component<IconComponentProps, {}, "">;
type BluetoothSlash = ReturnType<typeof BluetoothSlash>;
export default BluetoothSlash;
