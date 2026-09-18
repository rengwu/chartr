import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCircleGearIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCircleGear color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCircleGear: Component<IconComponentProps, {}, "">;
type UserCircleGear = ReturnType<typeof UserCircleGear>;
export default UserCircleGear;
