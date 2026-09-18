import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PintGlassIcon` instead.
 *
 * @example
 * ```svelte
 * <PintGlass color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PintGlass: Component<IconComponentProps, {}, "">;
type PintGlass = ReturnType<typeof PintGlass>;
export default PintGlass;
