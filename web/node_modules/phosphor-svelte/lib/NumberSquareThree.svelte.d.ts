import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareThree: Component<IconComponentProps, {}, "">;
type NumberSquareThree = ReturnType<typeof NumberSquareThree>;
export default NumberSquareThree;
