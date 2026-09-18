import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendUpLeft: Component<IconComponentProps, {}, "">;
type ArrowBendUpLeft = ReturnType<typeof ArrowBendUpLeft>;
export default ArrowBendUpLeft;
