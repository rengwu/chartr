import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlusMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <PlusMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlusMinus: Component<IconComponentProps, {}, "">;
type PlusMinus = ReturnType<typeof PlusMinus>;
export default PlusMinus;
