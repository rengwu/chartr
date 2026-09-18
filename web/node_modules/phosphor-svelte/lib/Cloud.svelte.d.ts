import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudIcon` instead.
 *
 * @example
 * ```svelte
 * <Cloud color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cloud: Component<IconComponentProps, {}, "">;
type Cloud = ReturnType<typeof Cloud>;
export default Cloud;
