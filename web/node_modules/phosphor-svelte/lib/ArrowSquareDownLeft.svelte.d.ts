import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareDownLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareDownLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareDownLeft: Component<IconComponentProps, {}, "">;
type ArrowSquareDownLeft = ReturnType<typeof ArrowSquareDownLeft>;
export default ArrowSquareDownLeft;
