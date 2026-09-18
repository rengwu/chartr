import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowUDownLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowUDownLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowUDownLeft: Component<IconComponentProps, {}, "">;
type ArrowUDownLeft = ReturnType<typeof ArrowUDownLeft>;
export default ArrowUDownLeft;
