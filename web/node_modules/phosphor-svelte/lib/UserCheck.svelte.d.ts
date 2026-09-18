import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCheck: Component<IconComponentProps, {}, "">;
type UserCheck = ReturnType<typeof UserCheck>;
export default UserCheck;
