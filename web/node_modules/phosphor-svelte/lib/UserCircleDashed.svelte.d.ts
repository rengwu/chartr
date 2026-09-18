import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCircleDashedIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCircleDashed color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCircleDashed: Component<IconComponentProps, {}, "">;
type UserCircleDashed = ReturnType<typeof UserCircleDashed>;
export default UserCircleDashed;
