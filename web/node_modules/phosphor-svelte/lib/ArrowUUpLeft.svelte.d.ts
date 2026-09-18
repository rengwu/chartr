import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowUUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowUUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowUUpLeft: Component<IconComponentProps, {}, "">;
type ArrowUUpLeft = ReturnType<typeof ArrowUUpLeft>;
export default ArrowUUpLeft;
