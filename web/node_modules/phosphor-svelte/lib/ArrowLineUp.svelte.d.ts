import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLineUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLineUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineUp: Component<IconComponentProps, {}, "">;
type ArrowLineUp = ReturnType<typeof ArrowLineUp>;
export default ArrowLineUp;
