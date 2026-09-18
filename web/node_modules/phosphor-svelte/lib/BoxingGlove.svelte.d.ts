import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoxingGloveIcon` instead.
 *
 * @example
 * ```svelte
 * <BoxingGlove color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BoxingGlove: Component<IconComponentProps, {}, "">;
type BoxingGlove = ReturnType<typeof BoxingGlove>;
export default BoxingGlove;
