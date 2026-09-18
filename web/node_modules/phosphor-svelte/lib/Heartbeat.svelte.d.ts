import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeartbeatIcon` instead.
 *
 * @example
 * ```svelte
 * <Heartbeat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Heartbeat: Component<IconComponentProps, {}, "">;
type Heartbeat = ReturnType<typeof Heartbeat>;
export default Heartbeat;
