import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <UserMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserMinus: Component<IconComponentProps, {}, "">;
type UserMinus = ReturnType<typeof UserMinus>;
export default UserMinus;
