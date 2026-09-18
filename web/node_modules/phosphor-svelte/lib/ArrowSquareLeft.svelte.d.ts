import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareLeft: Component<IconComponentProps, {}, "">;
type ArrowSquareLeft = ReturnType<typeof ArrowSquareLeft>;
export default ArrowSquareLeft;
