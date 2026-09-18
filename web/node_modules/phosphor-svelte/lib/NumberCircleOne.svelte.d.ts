import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleOneIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleOne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleOne: Component<IconComponentProps, {}, "">;
type NumberCircleOne = ReturnType<typeof NumberCircleOne>;
export default NumberCircleOne;
