import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MaskHappyIcon` instead.
 *
 * @example
 * ```svelte
 * <MaskHappy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MaskHappy: Component<IconComponentProps, {}, "">;
type MaskHappy = ReturnType<typeof MaskHappy>;
export default MaskHappy;
