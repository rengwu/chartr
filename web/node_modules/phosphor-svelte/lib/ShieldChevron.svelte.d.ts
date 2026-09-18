import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldChevronIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldChevron color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldChevron: Component<IconComponentProps, {}, "">;
type ShieldChevron = ReturnType<typeof ShieldChevron>;
export default ShieldChevron;
