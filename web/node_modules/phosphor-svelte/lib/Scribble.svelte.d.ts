import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScribbleIcon` instead.
 *
 * @example
 * ```svelte
 * <Scribble color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Scribble: Component<IconComponentProps, {}, "">;
type Scribble = ReturnType<typeof Scribble>;
export default Scribble;
