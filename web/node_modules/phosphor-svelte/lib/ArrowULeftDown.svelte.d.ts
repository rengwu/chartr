import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowULeftDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowULeftDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowULeftDown: Component<IconComponentProps, {}, "">;
type ArrowULeftDown = ReturnType<typeof ArrowULeftDown>;
export default ArrowULeftDown;
