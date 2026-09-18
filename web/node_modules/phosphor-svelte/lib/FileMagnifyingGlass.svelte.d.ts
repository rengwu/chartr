import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileMagnifyingGlassIcon` instead.
 *
 * @example
 * ```svelte
 * <FileMagnifyingGlass color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileMagnifyingGlass: Component<IconComponentProps, {}, "">;
type FileMagnifyingGlass = ReturnType<typeof FileMagnifyingGlass>;
export default FileMagnifyingGlass;
