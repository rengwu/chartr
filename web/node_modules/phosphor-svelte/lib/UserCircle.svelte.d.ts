import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCircle: Component<IconComponentProps, {}, "">;
type UserCircle = ReturnType<typeof UserCircle>;
export default UserCircle;
