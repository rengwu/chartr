import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ListMagnifyingGlassIcon` instead.
 *
 * @example
 * ```svelte
 * <ListMagnifyingGlass color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListMagnifyingGlass: Component<IconComponentProps, {}, "">;
type ListMagnifyingGlass = ReturnType<typeof ListMagnifyingGlass>;
export default ListMagnifyingGlass;
