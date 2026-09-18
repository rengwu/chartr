import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TabsIcon` instead.
 *
 * @example
 * ```svelte
 * <Tabs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tabs: Component<IconComponentProps, {}, "">;
type Tabs = ReturnType<typeof Tabs>;
export default Tabs;
