import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserSoundIcon` instead.
 *
 * @example
 * ```svelte
 * <UserSound color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserSound: Component<IconComponentProps, {}, "">;
type UserSound = ReturnType<typeof UserSound>;
export default UserSound;
