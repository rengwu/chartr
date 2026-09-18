import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClockUserIcon` instead.
 *
 * @example
 * ```svelte
 * <ClockUser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClockUser: Component<IconComponentProps, {}, "">;
type ClockUser = ReturnType<typeof ClockUser>;
export default ClockUser;
