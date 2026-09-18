import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlugsIcon` instead.
 *
 * @example
 * ```svelte
 * <Plugs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Plugs: Component<IconComponentProps, {}, "">;
type Plugs = ReturnType<typeof Plugs>;
export default Plugs;
