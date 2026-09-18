import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LayoutIcon` instead.
 *
 * @example
 * ```svelte
 * <Layout color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Layout: Component<IconComponentProps, {}, "">;
type Layout = ReturnType<typeof Layout>;
export default Layout;
