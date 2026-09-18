import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SpinnerBallIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpinnerBallIcon: Component<IconComponentProps, {}, "">;
type SpinnerBallIcon = ReturnType<typeof SpinnerBallIcon>;
export default SpinnerBallIcon;
