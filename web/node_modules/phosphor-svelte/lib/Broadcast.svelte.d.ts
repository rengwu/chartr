import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BroadcastIcon` instead.
 *
 * @example
 * ```svelte
 * <Broadcast color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Broadcast: Component<IconComponentProps, {}, "">;
type Broadcast = ReturnType<typeof Broadcast>;
export default Broadcast;
