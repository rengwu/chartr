import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowDownLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowDownLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowDownLeft: Component<IconComponentProps, {}, "">;
type ArrowDownLeft = ReturnType<typeof ArrowDownLeft>;
export default ArrowDownLeft;
