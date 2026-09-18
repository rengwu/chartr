import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BracketsAngleIcon` instead.
 *
 * @example
 * ```svelte
 * <BracketsAngle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BracketsAngle: Component<IconComponentProps, {}, "">;
type BracketsAngle = ReturnType<typeof BracketsAngle>;
export default BracketsAngle;
