import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FootballIcon` instead.
 *
 * @example
 * ```svelte
 * <Football color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Football: Component<IconComponentProps, {}, "">;
type Football = ReturnType<typeof Football>;
export default Football;
