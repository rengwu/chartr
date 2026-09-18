import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClockAfternoonIcon` instead.
 *
 * @example
 * ```svelte
 * <ClockAfternoon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClockAfternoon: Component<IconComponentProps, {}, "">;
type ClockAfternoon = ReturnType<typeof ClockAfternoon>;
export default ClockAfternoon;
