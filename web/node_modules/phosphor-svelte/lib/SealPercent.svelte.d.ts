import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SealPercentIcon` instead.
 *
 * @example
 * ```svelte
 * <SealPercent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SealPercent: Component<IconComponentProps, {}, "">;
type SealPercent = ReturnType<typeof SealPercent>;
export default SealPercent;
