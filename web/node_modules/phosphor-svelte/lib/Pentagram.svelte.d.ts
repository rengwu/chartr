import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PentagramIcon` instead.
 *
 * @example
 * ```svelte
 * <Pentagram color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pentagram: Component<IconComponentProps, {}, "">;
type Pentagram = ReturnType<typeof Pentagram>;
export default Pentagram;
