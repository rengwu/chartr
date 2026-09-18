import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TildeIcon` instead.
 *
 * @example
 * ```svelte
 * <Tilde color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tilde: Component<IconComponentProps, {}, "">;
type Tilde = ReturnType<typeof Tilde>;
export default Tilde;
