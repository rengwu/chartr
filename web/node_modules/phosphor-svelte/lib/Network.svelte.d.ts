import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NetworkIcon` instead.
 *
 * @example
 * ```svelte
 * <Network color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Network: Component<IconComponentProps, {}, "">;
type Network = ReturnType<typeof Network>;
export default Network;
