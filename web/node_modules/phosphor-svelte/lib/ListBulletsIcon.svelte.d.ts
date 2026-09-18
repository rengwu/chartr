import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ListBulletsIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListBulletsIcon: Component<IconComponentProps, {}, "">;
type ListBulletsIcon = ReturnType<typeof ListBulletsIcon>;
export default ListBulletsIcon;
