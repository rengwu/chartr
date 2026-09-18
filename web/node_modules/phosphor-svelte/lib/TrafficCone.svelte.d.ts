import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrafficConeIcon` instead.
 *
 * @example
 * ```svelte
 * <TrafficCone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrafficCone: Component<IconComponentProps, {}, "">;
type TrafficCone = ReturnType<typeof TrafficCone>;
export default TrafficCone;
