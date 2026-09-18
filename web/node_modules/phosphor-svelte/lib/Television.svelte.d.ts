import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TelevisionIcon` instead.
 *
 * @example
 * ```svelte
 * <Television color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Television: Component<IconComponentProps, {}, "">;
type Television = ReturnType<typeof Television>;
export default Television;
