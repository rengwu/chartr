import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PipeIcon` instead.
 *
 * @example
 * ```svelte
 * <Pipe color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pipe: Component<IconComponentProps, {}, "">;
type Pipe = ReturnType<typeof Pipe>;
export default Pipe;
