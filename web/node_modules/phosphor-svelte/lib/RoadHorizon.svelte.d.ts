import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RoadHorizonIcon` instead.
 *
 * @example
 * ```svelte
 * <RoadHorizon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RoadHorizon: Component<IconComponentProps, {}, "">;
type RoadHorizon = ReturnType<typeof RoadHorizon>;
export default RoadHorizon;
