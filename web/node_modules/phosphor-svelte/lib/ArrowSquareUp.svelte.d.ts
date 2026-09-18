import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareUp: Component<IconComponentProps, {}, "">;
type ArrowSquareUp = ReturnType<typeof ArrowSquareUp>;
export default ArrowSquareUp;
