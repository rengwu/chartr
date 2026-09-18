import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <UmbrellaSimpleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UmbrellaSimpleIcon: Component<IconComponentProps, {}, "">;
type UmbrellaSimpleIcon = ReturnType<typeof UmbrellaSimpleIcon>;
export default UmbrellaSimpleIcon;
