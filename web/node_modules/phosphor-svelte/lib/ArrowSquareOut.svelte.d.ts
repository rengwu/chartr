import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareOutIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareOut color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareOut: Component<IconComponentProps, {}, "">;
type ArrowSquareOut = ReturnType<typeof ArrowSquareOut>;
export default ArrowSquareOut;
