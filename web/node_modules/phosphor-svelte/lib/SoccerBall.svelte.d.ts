import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SoccerBallIcon` instead.
 *
 * @example
 * ```svelte
 * <SoccerBall color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SoccerBall: Component<IconComponentProps, {}, "">;
type SoccerBall = ReturnType<typeof SoccerBall>;
export default SoccerBall;
