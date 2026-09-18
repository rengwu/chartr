import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleFourIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleFour: Component<IconComponentProps, {}, "">;
type NumberCircleFour = ReturnType<typeof NumberCircleFour>;
export default NumberCircleFour;
