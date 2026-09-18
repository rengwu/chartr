import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SlidersHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <SlidersHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SlidersHorizontal: Component<IconComponentProps, {}, "">;
type SlidersHorizontal = ReturnType<typeof SlidersHorizontal>;
export default SlidersHorizontal;
