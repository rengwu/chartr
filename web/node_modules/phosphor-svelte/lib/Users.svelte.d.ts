import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UsersIcon` instead.
 *
 * @example
 * ```svelte
 * <Users color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Users: Component<IconComponentProps, {}, "">;
type Users = ReturnType<typeof Users>;
export default Users;
