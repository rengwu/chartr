import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <TelevisionSimpleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TelevisionSimpleIcon: Component<IconComponentProps, {}, "">;
type TelevisionSimpleIcon = ReturnType<typeof TelevisionSimpleIcon>;
export default TelevisionSimpleIcon;
