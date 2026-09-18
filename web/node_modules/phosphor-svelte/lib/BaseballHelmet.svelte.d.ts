import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BaseballHelmetIcon` instead.
 *
 * @example
 * ```svelte
 * <BaseballHelmet color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BaseballHelmet: Component<IconComponentProps, {}, "">;
type BaseballHelmet = ReturnType<typeof BaseballHelmet>;
export default BaseballHelmet;
