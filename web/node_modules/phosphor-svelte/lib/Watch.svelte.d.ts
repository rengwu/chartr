import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WatchIcon` instead.
 *
 * @example
 * ```svelte
 * <Watch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Watch: Component<IconComponentProps, {}, "">;
type Watch = ReturnType<typeof Watch>;
export default Watch;
