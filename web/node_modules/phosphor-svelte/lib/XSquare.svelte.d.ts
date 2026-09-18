import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `XSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <XSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const XSquare: Component<IconComponentProps, {}, "">;
type XSquare = ReturnType<typeof XSquare>;
export default XSquare;
