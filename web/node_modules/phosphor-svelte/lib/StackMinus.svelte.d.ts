import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StackMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <StackMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StackMinus: Component<IconComponentProps, {}, "">;
type StackMinus = ReturnType<typeof StackMinus>;
export default StackMinus;
