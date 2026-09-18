import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GpsSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <GpsSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GpsSlash: Component<IconComponentProps, {}, "">;
type GpsSlash = ReturnType<typeof GpsSlash>;
export default GpsSlash;
