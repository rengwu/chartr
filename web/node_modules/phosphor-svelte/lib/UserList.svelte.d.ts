import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserListIcon` instead.
 *
 * @example
 * ```svelte
 * <UserList color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserList: Component<IconComponentProps, {}, "">;
type UserList = ReturnType<typeof UserList>;
export default UserList;
