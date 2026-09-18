import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LetterCircleVIcon` instead.
 *
 * @example
 * ```svelte
 * <LetterCircleV color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LetterCircleV: Component<IconComponentProps, {}, "">;
type LetterCircleV = ReturnType<typeof LetterCircleV>;
export default LetterCircleV;
