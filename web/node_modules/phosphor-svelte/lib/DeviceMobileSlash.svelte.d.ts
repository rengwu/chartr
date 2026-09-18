import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeviceMobileSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <DeviceMobileSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceMobileSlash: Component<IconComponentProps, {}, "">;
type DeviceMobileSlash = ReturnType<typeof DeviceMobileSlash>;
export default DeviceMobileSlash;
