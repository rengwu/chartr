import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatDown: Component<IconComponentProps, {}, "">;
type ArrowFatDown = ReturnType<typeof ArrowFatDown>;
export default ArrowFatDown;
