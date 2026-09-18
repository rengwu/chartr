import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLinesLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLinesLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLinesLeft: Component<IconComponentProps, {}, "">;
type ArrowFatLinesLeft = ReturnType<typeof ArrowFatLinesLeft>;
export default ArrowFatLinesLeft;
