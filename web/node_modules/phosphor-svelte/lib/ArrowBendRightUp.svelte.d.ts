import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendRightUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendRightUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendRightUp: Component<IconComponentProps, {}, "">;
type ArrowBendRightUp = ReturnType<typeof ArrowBendRightUp>;
export default ArrowBendRightUp;
