import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToolboxIcon` instead.
 *
 * @example
 * ```svelte
 * <Toolbox color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Toolbox: Component<IconComponentProps, {}, "">;
type Toolbox = ReturnType<typeof Toolbox>;
export default Toolbox;
