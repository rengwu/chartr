import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLineUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLineUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineUpLeft: Component<IconComponentProps, {}, "">;
type ArrowLineUpLeft = ReturnType<typeof ArrowLineUpLeft>;
export default ArrowLineUpLeft;
