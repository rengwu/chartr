import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <CheckSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CheckSquare: Component<IconComponentProps, {}, "">;
type CheckSquare = ReturnType<typeof CheckSquare>;
export default CheckSquare;
