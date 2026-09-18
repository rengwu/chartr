import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeviceTabletSpeakerIcon` instead.
 *
 * @example
 * ```svelte
 * <DeviceTabletSpeaker color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceTabletSpeaker: Component<IconComponentProps, {}, "">;
type DeviceTabletSpeaker = ReturnType<typeof DeviceTabletSpeaker>;
export default DeviceTabletSpeaker;
