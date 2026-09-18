import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNoteIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNote color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNote: Component<IconComponentProps, {}, "">;
type MusicNote = ReturnType<typeof MusicNote>;
export default MusicNote;
