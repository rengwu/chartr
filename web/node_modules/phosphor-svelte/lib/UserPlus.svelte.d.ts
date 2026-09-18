import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <UserPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserPlus: Component<IconComponentProps, {}, "">;
type UserPlus = ReturnType<typeof UserPlus>;
export default UserPlus;
