import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SteeringWheelIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SteeringWheelIcon: Component<IconComponentProps, {}, "">;
type SteeringWheelIcon = ReturnType<typeof SteeringWheelIcon>;
export default SteeringWheelIcon;
