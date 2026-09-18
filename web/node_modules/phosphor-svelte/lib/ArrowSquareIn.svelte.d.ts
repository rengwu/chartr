import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareInIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareIn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareIn: Component<IconComponentProps, {}, "">;
type ArrowSquareIn = ReturnType<typeof ArrowSquareIn>;
export default ArrowSquareIn;
