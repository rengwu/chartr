import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserCircleCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <UserCircleCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserCircleCheck: Component<IconComponentProps, {}, "">;
type UserCircleCheck = ReturnType<typeof UserCircleCheck>;
export default UserCircleCheck;
