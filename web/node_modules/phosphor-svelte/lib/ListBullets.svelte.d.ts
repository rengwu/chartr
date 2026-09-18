import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ListBulletsIcon` instead.
 *
 * @example
 * ```svelte
 * <ListBullets color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListBullets: Component<IconComponentProps, {}, "">;
type ListBullets = ReturnType<typeof ListBullets>;
export default ListBullets;
