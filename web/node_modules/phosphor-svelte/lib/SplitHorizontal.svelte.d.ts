import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SplitHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <SplitHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SplitHorizontal: Component<IconComponentProps, {}, "">;
type SplitHorizontal = ReturnType<typeof SplitHorizontal>;
export default SplitHorizontal;
