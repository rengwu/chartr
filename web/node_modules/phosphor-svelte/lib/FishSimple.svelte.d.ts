import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FishSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <FishSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FishSimple: Component<IconComponentProps, {}, "">;
type FishSimple = ReturnType<typeof FishSimple>;
export default FishSimple;
