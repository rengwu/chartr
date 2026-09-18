import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserIcon` instead.
 *
 * @example
 * ```svelte
 * <User color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const User: Component<IconComponentProps, {}, "">;
type User = ReturnType<typeof User>;
export default User;
