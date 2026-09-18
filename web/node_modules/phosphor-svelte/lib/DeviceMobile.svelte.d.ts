import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeviceMobileIcon` instead.
 *
 * @example
 * ```svelte
 * <DeviceMobile color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceMobile: Component<IconComponentProps, {}, "">;
type DeviceMobile = ReturnType<typeof DeviceMobile>;
export default DeviceMobile;
