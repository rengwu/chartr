import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CrosshairSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <CrosshairSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CrosshairSimple: Component<IconComponentProps, {}, "">;
type CrosshairSimple = ReturnType<typeof CrosshairSimple>;
export default CrosshairSimple;
