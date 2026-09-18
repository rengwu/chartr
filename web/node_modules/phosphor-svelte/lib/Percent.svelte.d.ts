import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PercentIcon` instead.
 *
 * @example
 * ```svelte
 * <Percent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Percent: Component<IconComponentProps, {}, "">;
type Percent = ReturnType<typeof Percent>;
export default Percent;
