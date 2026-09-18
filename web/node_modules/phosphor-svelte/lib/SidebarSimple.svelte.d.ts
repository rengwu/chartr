import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SidebarSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <SidebarSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SidebarSimple: Component<IconComponentProps, {}, "">;
type SidebarSimple = ReturnType<typeof SidebarSimple>;
export default SidebarSimple;
