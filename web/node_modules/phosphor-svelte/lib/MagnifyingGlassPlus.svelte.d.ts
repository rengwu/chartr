import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MagnifyingGlassPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <MagnifyingGlassPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MagnifyingGlassPlus: Component<IconComponentProps, {}, "">;
type MagnifyingGlassPlus = ReturnType<typeof MagnifyingGlassPlus>;
export default MagnifyingGlassPlus;
