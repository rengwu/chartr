import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrafficSignalIcon` instead.
 *
 * @example
 * ```svelte
 * <TrafficSignal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrafficSignal: Component<IconComponentProps, {}, "">;
type TrafficSignal = ReturnType<typeof TrafficSignal>;
export default TrafficSignal;
