import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendDownLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendDownLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendDownLeft: Component<IconComponentProps, {}, "">;
type ArrowBendDownLeft = ReturnType<typeof ArrowBendDownLeft>;
export default ArrowBendDownLeft;
