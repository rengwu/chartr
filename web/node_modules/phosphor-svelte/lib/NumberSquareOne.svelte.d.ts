import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareOneIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareOne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareOne: Component<IconComponentProps, {}, "">;
type NumberSquareOne = ReturnType<typeof NumberSquareOne>;
export default NumberSquareOne;
