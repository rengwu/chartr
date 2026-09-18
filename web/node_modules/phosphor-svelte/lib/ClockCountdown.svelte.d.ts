import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClockCountdownIcon` instead.
 *
 * @example
 * ```svelte
 * <ClockCountdown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClockCountdown: Component<IconComponentProps, {}, "">;
type ClockCountdown = ReturnType<typeof ClockCountdown>;
export default ClockCountdown;
