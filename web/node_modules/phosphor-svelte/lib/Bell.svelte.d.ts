import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellIcon` instead.
 *
 * @example
 * ```svelte
 * <Bell color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bell: Component<IconComponentProps, {}, "">;
type Bell = ReturnType<typeof Bell>;
export default Bell;
