import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UmbrellaSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <UmbrellaSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UmbrellaSimple: Component<IconComponentProps, {}, "">;
type UmbrellaSimple = ReturnType<typeof UmbrellaSimple>;
export default UmbrellaSimple;
