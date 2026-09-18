import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLeft: Component<IconComponentProps, {}, "">;
type ArrowFatLeft = ReturnType<typeof ArrowFatLeft>;
export default ArrowFatLeft;
