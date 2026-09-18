import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GenderTransgenderIcon` instead.
 *
 * @example
 * ```svelte
 * <GenderTransgender color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GenderTransgender: Component<IconComponentProps, {}, "">;
type GenderTransgender = ReturnType<typeof GenderTransgender>;
export default GenderTransgender;
