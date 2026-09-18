import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsSplitIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsSplit color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsSplit: Component<IconComponentProps, {}, "">;
type ArrowsSplit = ReturnType<typeof ArrowsSplit>;
export default ArrowsSplit;
