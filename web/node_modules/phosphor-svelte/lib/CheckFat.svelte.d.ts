import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckFatIcon` instead.
 *
 * @example
 * ```svelte
 * <CheckFat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CheckFat: Component<IconComponentProps, {}, "">;
type CheckFat = ReturnType<typeof CheckFat>;
export default CheckFat;
