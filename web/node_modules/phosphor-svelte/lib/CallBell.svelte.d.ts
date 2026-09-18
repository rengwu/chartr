import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CallBellIcon` instead.
 *
 * @example
 * ```svelte
 * <CallBell color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CallBell: Component<IconComponentProps, {}, "">;
type CallBell = ReturnType<typeof CallBell>;
export default CallBell;
