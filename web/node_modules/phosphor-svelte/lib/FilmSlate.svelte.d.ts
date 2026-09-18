import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilmSlateIcon` instead.
 *
 * @example
 * ```svelte
 * <FilmSlate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilmSlate: Component<IconComponentProps, {}, "">;
type FilmSlate = ReturnType<typeof FilmSlate>;
export default FilmSlate;
