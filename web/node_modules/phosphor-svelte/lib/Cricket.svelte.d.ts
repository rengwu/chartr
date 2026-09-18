import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CricketIcon` instead.
 *
 * @example
 * ```svelte
 * <Cricket color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cricket: Component<IconComponentProps, {}, "">;
type Cricket = ReturnType<typeof Cricket>;
export default Cricket;
