import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RssSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <RssSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RssSimple: Component<IconComponentProps, {}, "">;
type RssSimple = ReturnType<typeof RssSimple>;
export default RssSimple;
