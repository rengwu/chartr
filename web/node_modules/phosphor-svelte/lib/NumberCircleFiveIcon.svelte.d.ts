import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <NumberCircleFiveIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleFiveIcon: Component<IconComponentProps, {}, "">;
type NumberCircleFiveIcon = ReturnType<typeof NumberCircleFiveIcon>;
export default NumberCircleFiveIcon;
