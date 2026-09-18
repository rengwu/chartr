import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OrangeIcon` instead.
 *
 * @example
 * ```svelte
 * <Orange color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Orange: Component<IconComponentProps, {}, "">;
type Orange = ReturnType<typeof Orange>;
export default Orange;
