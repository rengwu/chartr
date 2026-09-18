import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MinusSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <MinusSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MinusSquare: Component<IconComponentProps, {}, "">;
type MinusSquare = ReturnType<typeof MinusSquare>;
export default MinusSquare;
