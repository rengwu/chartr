import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TornadoIcon` instead.
 *
 * @example
 * ```svelte
 * <Tornado color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tornado: Component<IconComponentProps, {}, "">;
type Tornado = ReturnType<typeof Tornado>;
export default Tornado;
