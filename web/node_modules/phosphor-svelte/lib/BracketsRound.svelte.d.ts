import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BracketsRoundIcon` instead.
 *
 * @example
 * ```svelte
 * <BracketsRound color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BracketsRound: Component<IconComponentProps, {}, "">;
type BracketsRound = ReturnType<typeof BracketsRound>;
export default BracketsRound;
