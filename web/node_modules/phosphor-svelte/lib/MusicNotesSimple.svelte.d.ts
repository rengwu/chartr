import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNotesSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNotesSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNotesSimple: Component<IconComponentProps, {}, "">;
type MusicNotesSimple = ReturnType<typeof MusicNotesSimple>;
export default MusicNotesSimple;
