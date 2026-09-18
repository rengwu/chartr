import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CardsIcon` instead.
 *
 * @example
 * ```svelte
 * <Cards color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cards: Component<IconComponentProps, {}, "">;
type Cards = ReturnType<typeof Cards>;
export default Cards;
