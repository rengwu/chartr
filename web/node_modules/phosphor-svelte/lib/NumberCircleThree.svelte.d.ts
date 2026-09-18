import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleThree: Component<IconComponentProps, {}, "">;
type NumberCircleThree = ReturnType<typeof NumberCircleThree>;
export default NumberCircleThree;
