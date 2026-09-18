import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AsteriskIcon` instead.
 *
 * @example
 * ```svelte
 * <Asterisk color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Asterisk: Component<IconComponentProps, {}, "">;
type Asterisk = ReturnType<typeof Asterisk>;
export default Asterisk;
