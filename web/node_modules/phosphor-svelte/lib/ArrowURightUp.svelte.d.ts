import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowURightUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowURightUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowURightUp: Component<IconComponentProps, {}, "">;
type ArrowURightUp = ReturnType<typeof ArrowURightUp>;
export default ArrowURightUp;
