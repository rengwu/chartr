import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LogIcon` instead.
 *
 * @example
 * ```svelte
 * <Log color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Log: Component<IconComponentProps, {}, "">;
type Log = ReturnType<typeof Log>;
export default Log;
