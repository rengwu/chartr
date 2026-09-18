import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNotesMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNotesMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNotesMinus: Component<IconComponentProps, {}, "">;
type MusicNotesMinus = ReturnType<typeof MusicNotesMinus>;
export default MusicNotesMinus;
