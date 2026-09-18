import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GpsFixIcon` instead.
 *
 * @example
 * ```svelte
 * <GpsFix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GpsFix: Component<IconComponentProps, {}, "">;
type GpsFix = ReturnType<typeof GpsFix>;
export default GpsFix;
