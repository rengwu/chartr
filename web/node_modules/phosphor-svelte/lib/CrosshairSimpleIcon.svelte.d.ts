import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CrosshairSimpleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CrosshairSimpleIcon: Component<IconComponentProps, {}, "">;
type CrosshairSimpleIcon = ReturnType<typeof CrosshairSimpleIcon>;
export default CrosshairSimpleIcon;
