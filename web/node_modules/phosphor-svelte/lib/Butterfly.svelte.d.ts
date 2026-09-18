import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ButterflyIcon` instead.
 *
 * @example
 * ```svelte
 * <Butterfly color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Butterfly: Component<IconComponentProps, {}, "">;
type Butterfly = ReturnType<typeof Butterfly>;
export default Butterfly;
