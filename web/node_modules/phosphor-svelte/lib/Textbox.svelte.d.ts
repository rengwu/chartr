import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextboxIcon` instead.
 *
 * @example
 * ```svelte
 * <Textbox color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Textbox: Component<IconComponentProps, {}, "">;
type Textbox = ReturnType<typeof Textbox>;
export default Textbox;
