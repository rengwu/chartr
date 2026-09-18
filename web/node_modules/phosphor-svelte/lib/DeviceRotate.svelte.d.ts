import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeviceRotateIcon` instead.
 *
 * @example
 * ```svelte
 * <DeviceRotate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceRotate: Component<IconComponentProps, {}, "">;
type DeviceRotate = ReturnType<typeof DeviceRotate>;
export default DeviceRotate;
