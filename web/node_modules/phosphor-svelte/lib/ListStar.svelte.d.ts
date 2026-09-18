import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ListStarIcon` instead.
 *
 * @example
 * ```svelte
 * <ListStar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListStar: Component<IconComponentProps, {}, "">;
type ListStar = ReturnType<typeof ListStar>;
export default ListStar;
