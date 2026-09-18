import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleSevenIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleSeven color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleSeven: Component<IconComponentProps, {}, "">;
type NumberCircleSeven = ReturnType<typeof NumberCircleSeven>;
export default NumberCircleSeven;
