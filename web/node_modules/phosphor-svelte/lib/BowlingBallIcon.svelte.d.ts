import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <BowlingBallIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BowlingBallIcon: Component<IconComponentProps, {}, "">;
type BowlingBallIcon = ReturnType<typeof BowlingBallIcon>;
export default BowlingBallIcon;
