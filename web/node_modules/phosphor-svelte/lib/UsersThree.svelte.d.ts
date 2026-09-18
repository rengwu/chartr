import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UsersThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <UsersThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UsersThree: Component<IconComponentProps, {}, "">;
type UsersThree = ReturnType<typeof UsersThree>;
export default UsersThree;
