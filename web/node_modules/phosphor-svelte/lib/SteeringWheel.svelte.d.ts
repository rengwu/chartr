import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SteeringWheelIcon` instead.
 *
 * @example
 * ```svelte
 * <SteeringWheel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SteeringWheel: Component<IconComponentProps, {}, "">;
type SteeringWheel = ReturnType<typeof SteeringWheel>;
export default SteeringWheel;
