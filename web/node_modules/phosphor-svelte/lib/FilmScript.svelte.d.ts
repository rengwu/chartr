import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilmScriptIcon` instead.
 *
 * @example
 * ```svelte
 * <FilmScript color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilmScript: Component<IconComponentProps, {}, "">;
type FilmScript = ReturnType<typeof FilmScript>;
export default FilmScript;
