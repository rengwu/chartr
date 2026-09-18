import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleTwo: Component<IconComponentProps, {}, "">;
type NumberCircleTwo = ReturnType<typeof NumberCircleTwo>;
export default NumberCircleTwo;
