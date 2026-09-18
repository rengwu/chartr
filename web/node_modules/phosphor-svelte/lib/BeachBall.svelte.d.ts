import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BeachBallIcon` instead.
 *
 * @example
 * ```svelte
 * <BeachBall color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BeachBall: Component<IconComponentProps, {}, "">;
type BeachBall = ReturnType<typeof BeachBall>;
export default BeachBall;
