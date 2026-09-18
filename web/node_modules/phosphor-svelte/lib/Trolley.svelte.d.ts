import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrolleyIcon` instead.
 *
 * @example
 * ```svelte
 * <Trolley color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Trolley: Component<IconComponentProps, {}, "">;
type Trolley = ReturnType<typeof Trolley>;
export default Trolley;
