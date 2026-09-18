import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileySadIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileySad color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileySad: Component<IconComponentProps, {}, "">;
type SmileySad = ReturnType<typeof SmileySad>;
export default SmileySad;
