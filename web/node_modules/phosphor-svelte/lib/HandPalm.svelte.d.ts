import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandPalmIcon` instead.
 *
 * @example
 * ```svelte
 * <HandPalm color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandPalm: Component<IconComponentProps, {}, "">;
type HandPalm = ReturnType<typeof HandPalm>;
export default HandPalm;
