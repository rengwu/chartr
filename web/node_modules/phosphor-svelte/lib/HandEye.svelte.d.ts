import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandEyeIcon` instead.
 *
 * @example
 * ```svelte
 * <HandEye color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandEye: Component<IconComponentProps, {}, "">;
type HandEye = ReturnType<typeof HandEye>;
export default HandEye;
