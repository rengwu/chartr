import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserGearIcon` instead.
 *
 * @example
 * ```svelte
 * <UserGear color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserGear: Component<IconComponentProps, {}, "">;
type UserGear = ReturnType<typeof UserGear>;
export default UserGear;
