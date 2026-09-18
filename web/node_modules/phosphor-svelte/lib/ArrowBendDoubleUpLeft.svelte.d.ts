import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendDoubleUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendDoubleUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendDoubleUpLeft: Component<IconComponentProps, {}, "">;
type ArrowBendDoubleUpLeft = ReturnType<typeof ArrowBendDoubleUpLeft>;
export default ArrowBendDoubleUpLeft;
