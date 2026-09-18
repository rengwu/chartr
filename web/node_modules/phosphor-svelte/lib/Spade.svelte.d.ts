import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpadeIcon` instead.
 *
 * @example
 * ```svelte
 * <Spade color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Spade: Component<IconComponentProps, {}, "">;
type Spade = ReturnType<typeof Spade>;
export default Spade;
