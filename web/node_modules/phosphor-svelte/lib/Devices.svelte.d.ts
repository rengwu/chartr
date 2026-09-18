import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DevicesIcon` instead.
 *
 * @example
 * ```svelte
 * <Devices color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Devices: Component<IconComponentProps, {}, "">;
type Devices = ReturnType<typeof Devices>;
export default Devices;
