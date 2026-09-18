import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilmStripIcon` instead.
 *
 * @example
 * ```svelte
 * <FilmStrip color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilmStrip: Component<IconComponentProps, {}, "">;
type FilmStrip = ReturnType<typeof FilmStrip>;
export default FilmStrip;
