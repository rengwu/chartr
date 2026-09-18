import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCirclePlusIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCirclePlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCirclePlus: Component<IconComponentProps, {}, "">;
type UserCirclePlus = ReturnType<typeof UserCirclePlus>;
export default UserCirclePlus;
