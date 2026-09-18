import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <DeviceTabletIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceTabletIcon: Component<IconComponentProps, {}, "">;
type DeviceTabletIcon = ReturnType<typeof DeviceTabletIcon>;
export default DeviceTabletIcon;
