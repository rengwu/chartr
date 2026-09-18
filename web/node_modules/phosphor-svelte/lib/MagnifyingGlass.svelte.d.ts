import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MagnifyingGlassIcon` instead.
 *
 * @example
 * ```svelte
 * <MagnifyingGlass color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MagnifyingGlass: Component<IconComponentProps, {}, "">;
type MagnifyingGlass = ReturnType<typeof MagnifyingGlass>;
export default MagnifyingGlass;
