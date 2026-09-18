import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareRight: Component<IconComponentProps, {}, "">;
type ArrowSquareRight = ReturnType<typeof ArrowSquareRight>;
export default ArrowSquareRight;
