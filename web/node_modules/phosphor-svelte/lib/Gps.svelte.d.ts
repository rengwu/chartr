import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GpsIcon` instead.
 *
 * @example
 * ```svelte
 * <Gps color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Gps: Component<IconComponentProps, {}, "">;
type Gps = ReturnType<typeof Gps>;
export default Gps;
