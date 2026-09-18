import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandshakeIcon` instead.
 *
 * @example
 * ```svelte
 * <Handshake color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Handshake: Component<IconComponentProps, {}, "">;
type Handshake = ReturnType<typeof Handshake>;
export default Handshake;
