import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserFocusIcon` instead.
 *
 * @example
 * ```svelte
 * <UserFocus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserFocus: Component<IconComponentProps, {}, "">;
type UserFocus = ReturnType<typeof UserFocus>;
export default UserFocus;
