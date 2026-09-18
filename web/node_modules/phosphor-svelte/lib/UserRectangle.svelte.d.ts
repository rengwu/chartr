import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UserRectangleIcon` instead.
 *
 * @example
 * ```svelte
 * <UserRectangle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UserRectangle: Component<IconComponentProps, {}, "">;
type UserRectangle = ReturnType<typeof UserRectangle>;
export default UserRectangle;
