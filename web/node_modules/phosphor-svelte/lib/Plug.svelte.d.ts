import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlugIcon` instead.
 *
 * @example
 * ```svelte
 * <Plug color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Plug: Component<IconComponentProps, {}, "">;
type Plug = ReturnType<typeof Plug>;
export default Plug;
