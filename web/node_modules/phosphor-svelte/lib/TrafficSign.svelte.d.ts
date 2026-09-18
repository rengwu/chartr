import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrafficSignIcon` instead.
 *
 * @example
 * ```svelte
 * <TrafficSign color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrafficSign: Component<IconComponentProps, {}, "">;
type TrafficSign = ReturnType<typeof TrafficSign>;
export default TrafficSign;
