import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BaseballCapIcon` instead.
 *
 * @example
 * ```svelte
 * <BaseballCap color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BaseballCap: Component<IconComponentProps, {}, "">;
type BaseballCap = ReturnType<typeof BaseballCap>;
export default BaseballCap;
