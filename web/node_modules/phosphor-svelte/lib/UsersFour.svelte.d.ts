import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UsersFourIcon` instead.
 *
 * @example
 * ```svelte
 * <UsersFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UsersFour: Component<IconComponentProps, {}, "">;
type UsersFour = ReturnType<typeof UsersFour>;
export default UsersFour;
