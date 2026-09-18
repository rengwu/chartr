import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BracketsCurlyIcon` instead.
 *
 * @example
 * ```svelte
 * <BracketsCurly color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BracketsCurly: Component<IconComponentProps, {}, "">;
type BracketsCurly = ReturnType<typeof BracketsCurly>;
export default BracketsCurly;
