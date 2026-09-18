import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserSwitchIcon` instead.
 *
 * @example
 * ```svelte
 * <UserSwitch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserSwitch: Component<IconComponentProps, {}, "">;
type UserSwitch = ReturnType<typeof UserSwitch>;
export default UserSwitch;
