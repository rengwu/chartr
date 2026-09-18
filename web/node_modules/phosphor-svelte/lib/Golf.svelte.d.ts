import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GolfIcon` instead.
 *
 * @example
 * ```svelte
 * <Golf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Golf: Component<IconComponentProps, {}, "">;
type Golf = ReturnType<typeof Golf>;
export default Golf;
