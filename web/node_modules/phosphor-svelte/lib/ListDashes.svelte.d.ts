import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ListDashesIcon` instead.
 *
 * @example
 * ```svelte
 * <ListDashes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListDashes: Component<IconComponentProps, {}, "">;
type ListDashes = ReturnType<typeof ListDashes>;
export default ListDashes;
