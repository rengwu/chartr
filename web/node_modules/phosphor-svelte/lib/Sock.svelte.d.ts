import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SockIcon` instead.
 *
 * @example
 * ```svelte
 * <Sock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sock: Component<IconComponentProps, {}, "">;
type Sock = ReturnType<typeof Sock>;
export default Sock;
