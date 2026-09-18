import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FarmIcon` instead.
 *
 * @example
 * ```svelte
 * <Farm color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Farm: Component<IconComponentProps, {}, "">;
type Farm = ReturnType<typeof Farm>;
export default Farm;
