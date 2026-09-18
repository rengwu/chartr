import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GaugeIcon` instead.
 *
 * @example
 * ```svelte
 * <Gauge color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Gauge: Component<IconComponentProps, {}, "">;
type Gauge = ReturnType<typeof Gauge>;
export default Gauge;
