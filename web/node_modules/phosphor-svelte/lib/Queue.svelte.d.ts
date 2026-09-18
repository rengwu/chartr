import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `QueueIcon` instead.
 *
 * @example
 * ```svelte
 * <Queue color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Queue: Component<IconComponentProps, {}, "">;
type Queue = ReturnType<typeof Queue>;
export default Queue;
