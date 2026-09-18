import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareUpLeft: Component<IconComponentProps, {}, "">;
type ArrowSquareUpLeft = ReturnType<typeof ArrowSquareUpLeft>;
export default ArrowSquareUpLeft;
